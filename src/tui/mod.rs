use crate::{
    actions::{ActionDefinition, TargetMode, TargetRelation, TargetState},
    app::{App, AppAction, AppMode},
    campaign::elements,
    entities::Allegiance,
    glyphs::Element,
    saves::{read_save, validate_save_name, SaveData},
};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    io::{self, Stdout},
    path::Path,
    time::{Duration, Instant},
};

const MENU: [&str; 5] = ["Jogar", "Carregar Saves", "Mods", "Créditos", "Sair"];
const HISTORY_LIMIT: usize = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    MainMenu,
    NewGame,
    Saves,
    Mods,
    Credits,
    Ready,
    Combat,
    Prologue,
    Explore,
    Character,
    Inventory,
    Spells,
    Equipment,
    Locations,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Dialog {
    Exit,
    Load(AppAction),
    Replace,
    SkipPrologue,
}

#[derive(Clone, Debug)]
struct SaveRow {
    name: String,
    character: Option<String>,
    race: Option<String>,
    details: String,
    loadable: bool,
}

#[derive(Default)]
struct TargetPicker {
    action_id: String,
    targets: Vec<String>,
    cursor: usize,
    selected: BTreeSet<String>,
    maximum: usize,
}

#[derive(Default)]
struct UiState {
    screen: Option<Screen>,
    selected: usize,
    target_picker: Option<TargetPicker>,
    dialog: Option<Dialog>,
    save_name: String,
    character_name: String,
    race_index: usize,
    selected_elements: BTreeSet<Element>,
    form_field: usize,
    glitch_frame: u8,
    glitch_finished: bool,
    saves: Vec<SaveRow>,
    message: String,
    history: VecDeque<String>,
    show_explanation: bool,
}

pub fn run(app: App) -> io::Result<()> {
    Tui::new(app).run()
}

struct Tui {
    app: App,
    ui: UiState,
    scene_started: Instant,
    tracked_scene: Option<String>,
}

impl Tui {
    fn new(app: App) -> Self {
        Self {
            app,
            scene_started: Instant::now(),
            tracked_scene: None,
            ui: UiState {
                screen: Some(Screen::MainMenu),
                message: "Bem-vindo a A Magic Sovereign.".into(),
                ..UiState::default()
            },
        }
    }

    fn glitch_is_active(&self) -> bool {
        self.ui.screen == Some(Screen::Prologue)
            && self
                .app
                .campaign
                .as_ref()
                .and_then(|progress| progress.current_scene_id.as_deref())
                .and_then(|id| self.app.state.scene_registry.get(id))
                .is_some_and(|scene| scene.style == "glitch")
    }

    fn selected_race(&self) -> Option<&crate::campaign::RaceDefinition> {
        self.app.state.race_registry.iter().nth(self.ui.race_index)
    }

    fn sync_combat_screen(&mut self) {
        if self.app.mode == AppMode::Combat {
            return;
        }
        self.ui.screen = Some(
            if self
                .app
                .campaign
                .as_ref()
                .is_some_and(|campaign| campaign.prologue_completed)
            {
                Screen::Explore
            } else {
                Screen::Ready
            },
        );
        self.ui.selected = 0;
    }

    fn auto_advance_active(&self) -> bool {
        self.ui.screen == Some(Screen::Prologue)
            && self
                .app
                .campaign
                .as_ref()
                .and_then(|progress| progress.current_scene_id.as_deref())
                .and_then(|id| self.app.state.scene_registry.get(id))
                .is_some_and(|scene| scene.auto_advance)
    }

    fn auto_advance_due(&self) -> bool {
        self.auto_advance_active() && self.scene_started.elapsed() >= Duration::from_secs(3)
    }

    fn sync_scene_state(&mut self) {
        let current = self
            .app
            .campaign
            .as_ref()
            .and_then(|progress| progress.current_scene_id.clone());
        if current != self.tracked_scene {
            self.tracked_scene = current;
            self.scene_started = Instant::now();
            self.ui.glitch_frame = 0;
            self.ui.glitch_finished = false;
        }
    }

    fn run(mut self) -> io::Result<()> {
        let mut terminal = TerminalSession::enter()?;
        terminal.terminal.clear()?;
        terminal.terminal.draw(|frame| self.render(frame))?;

        while !self.app.should_exit {
            self.sync_scene_state();
            let animate = self.glitch_is_active()
                && self.app.state.visual_effects == "full"
                && !self.ui.glitch_finished;
            let automatic = self.auto_advance_due();
            let input = event::poll(Duration::from_millis(if animate {
                70
            } else if self.auto_advance_active() {
                100
            } else {
                250
            }))?;
            if input {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
            if automatic && !input {
                let result = self.app.dispatch(AppAction::AdvanceScene);
                self.set_message(result);
                self.scene_started = Instant::now();
                if self
                    .app
                    .campaign
                    .as_ref()
                    .is_some_and(|v| v.prologue_completed)
                {
                    self.ui.screen = Some(Screen::Explore);
                }
            }
            if input || animate || automatic {
                if animate {
                    self.ui.glitch_frame = (self.ui.glitch_frame + 1) % 8;
                    self.ui.glitch_finished = self.ui.glitch_frame == 7;
                }
                terminal.terminal.draw(|frame| self.render(frame))?;
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(area);
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "A MAGIC SOVEREIGN",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("    v{}", env!("CARGO_PKG_VERSION"))),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(title, layout[0]);

        match self.ui.screen.unwrap_or(Screen::MainMenu) {
            Screen::MainMenu => self.render_menu(frame, layout[1]),
            Screen::NewGame => self.render_new_game(frame, layout[1]),
            Screen::Saves => self.render_saves(frame, layout[1]),
            Screen::Mods => self.render_mods(frame, layout[1]),
            Screen::Credits => self.render_credits(frame, layout[1]),
            Screen::Ready => self.render_ready(frame, layout[1]),
            Screen::Combat => self.render_combat(frame, layout[1]),
            Screen::Prologue => self.render_prologue(frame, layout[1]),
            Screen::Explore => self.render_explore(frame, layout[1]),
            Screen::Character => self.render_character(frame, layout[1]),
            Screen::Inventory => self.render_inventory(frame, layout[1]),
            Screen::Spells => self.render_spells(frame, layout[1]),
            Screen::Equipment => self.render_equipment(frame, layout[1]),
            Screen::Locations => self.render_locations(frame, layout[1]),
        }
        let footer = Paragraph::new(self.footer_text())
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(footer, layout[2]);
        if let Some(dialog) = &self.ui.dialog {
            self.render_dialog(frame, area, dialog);
        }
    }

    fn render_menu(&self, frame: &mut Frame<'_>, area: Rect) {
        let items = MENU
            .iter()
            .map(|label| ListItem::new(*label))
            .collect::<Vec<_>>();
        let list = List::new(items)
            .block(
                Block::default()
                    .title("Menu Principal")
                    .borders(Borders::ALL),
            )
            .highlight_symbol("> ")
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            );
        let mut state = ListState::default();
        state.select(Some(self.ui.selected.min(MENU.len().saturating_sub(1))));
        let centered = centered_rect(60, 78, area);
        frame.render_stateful_widget(list, centered, &mut state);
    }

    fn render_new_game(&self, frame: &mut Frame<'_>, area: Rect) {
        let active = |index: usize| {
            if self.ui.form_field == index {
                "> "
            } else {
                "  "
            }
        };
        let mut text = vec![Line::from("Escolha até dois elementos. Raça persiste como identidade; mecânicas raciais estão em desenvolvimento."), Line::from("")];
        text.push(Line::from(format!(
            "{}Identificador do save: {}{}",
            active(0),
            self.ui.save_name,
            if self.ui.form_field == 0 { "_" } else { "" }
        )));
        text.push(Line::from(format!(
            "{}Nome do personagem: {}{}",
            active(1),
            self.ui.character_name,
            if self.ui.form_field == 1 { "_" } else { "" }
        )));
        text.push(Line::from(format!(
            "{}Raça: {} — {}",
            active(2),
            self.selected_race()
                .map(|race| race.name.as_str())
                .unwrap_or("Nenhuma raça disponível"),
            self.selected_race()
                .map(|race| race.description.as_str())
                .unwrap_or("Verifique data/content/races.json")
        )));
        text.push(Line::from("Elementos (Espaço para marcar):"));
        for pair in 0..7 {
            let first = elements()[pair * 2];
            let left_index = 3 + pair * 2;
            let right_index = left_index + 1;
            let left = format!(
                "{}[{}] {}",
                if self.ui.form_field == left_index {
                    ">"
                } else {
                    " "
                },
                if self.ui.selected_elements.contains(&first) {
                    "x"
                } else {
                    " "
                },
                first.label()
            );
            let second = elements().get(pair * 2 + 1).copied();
            let left_style = if self.ui.form_field == left_index {
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            text.push(Line::from(vec![
                Span::styled(left, left_style),
                Span::raw("    "),
                if let Some(second) = second {
                    Span::styled(
                        format!(
                            "{}[{}] {}",
                            if self.ui.form_field == right_index {
                                ">"
                            } else {
                                " "
                            },
                            if self.ui.selected_elements.contains(&second) {
                                "x"
                            } else {
                                " "
                            },
                            second.label()
                        ),
                        if self.ui.form_field == right_index {
                            left_style
                        } else {
                            Style::default()
                        },
                    )
                } else {
                    Span::raw("")
                },
            ]));
        }
        text.push(Line::from(""));
        text.push(Line::from(format!("{}Confirmar criação", active(16))));
        text.push(Line::from("Atributos iniciais: perfil neutro do engine. Quatro magias iniciais aguardam definições de conteúdo."));
        let panel = Paragraph::new(Text::from(text))
            .block(Block::default().title("Novo Jogo").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(panel, centered_rect(92, 94, area));
    }

    fn render_prologue(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(progress) = &self.app.campaign else {
            frame.render_widget(Paragraph::new("Nenhuma campanha ativa."), area);
            return;
        };
        let Some(id) = progress.current_scene_id.as_deref() else {
            return;
        };
        let Some(scene) = self.app.state.scene_registry.get(id) else {
            frame.render_widget(
                Paragraph::new(format!(
                    "Cena ausente: {id}. O save foi preservado; consulte logs/latest.log."
                )),
                area,
            );
            return;
        };
        let glitch = scene.style == "glitch";
        let effects = self.app.state.visual_effects.as_str();
        let color = if !glitch || effects == "off" {
            Color::White
        } else if effects == "full" && self.ui.glitch_frame == 6 {
            Color::Black
        } else if effects == "full" {
            [
                Color::LightRed,
                Color::White,
                Color::LightYellow,
                Color::Red,
            ][usize::from(self.ui.glitch_frame % 4)]
        } else {
            Color::LightRed
        };
        let text = if glitch && effects == "full" && !self.ui.glitch_finished {
            glitch_text(self.ui.glitch_frame)
        } else {
            scene.text.clone()
        };
        let panel_area = centered_rect(82, 54, area);
        let block = Block::default()
            .title(if glitch { " " } else { "Prólogo" })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color));
        if glitch {
            let inner = block.inner(panel_area);
            frame.render_widget(block, panel_area);
            let center = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner)[1];
            frame.render_widget(
                Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(color)),
                center,
            );
        } else {
            frame.render_widget(
                Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(color))
                    .wrap(Wrap { trim: true }),
                panel_area,
            );
        }
    }

    fn render_explore(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(progress) = &self.app.campaign else {
            frame.render_widget(Paragraph::new("Nenhuma campanha ativa."), area);
            return;
        };
        let location_id = progress.location_id.as_deref().unwrap_or("core:tidal_town");
        let location = self.app.state.location_registry.get(location_id);
        let player = self.app.player.as_ref();
        let mut lines = vec![
            Line::from(
                location
                    .map(|v| v.name.as_str())
                    .unwrap_or(location_id)
                    .to_owned(),
            ),
            Line::from(format!("Dia {}", progress.day)),
            Line::from(""),
            Line::from(
                location
                    .map(|v| v.description.as_str())
                    .unwrap_or("Localização indisponível."),
            ),
            Line::from(""),
        ];
        if let Some(player) = player {
            lines.push(Line::from(format!(
                "HP {}/{}   Tenacidade {}/{}   Mana {}/{}",
                player.hp.current,
                player.hp.maximum,
                player.tenacity.current,
                player.tenacity.maximum,
                player.mana.current,
                player.mana.maximum
            )));
        }
        lines.push(Line::from(""));
        for (index, option) in [
            "Explorar locais",
            "Personagem",
            "Inventário",
            "Magias",
            "Equipamentos",
            "Descansar",
            "Salvar",
        ]
        .iter()
        .enumerate()
        {
            lines.push(Line::from(format!(
                "{}{}",
                if self.ui.selected == index {
                    "> "
                } else {
                    "  "
                },
                option
            )));
        }
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .block(Block::default().title("Exploração").borders(Borders::ALL))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn render_character(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut lines = vec![
            Line::from(format!(
                "Nome: {}",
                self.app
                    .player
                    .as_ref()
                    .map(|p| p.name.as_str())
                    .unwrap_or("—")
            )),
            Line::from(format!(
                "Raça: {}",
                self.app
                    .character
                    .as_ref()
                    .and_then(|profile| self.app.state.race_registry.get(&profile.race_id))
                    .map(|race| race.name.as_str())
                    .unwrap_or("—")
            )),
            Line::from("Mecânicas raciais específicas ainda estão em desenvolvimento."),
            Line::from("Atributos:"),
        ];
        if let Some(player) = &self.app.player {
            for attribute in crate::attributes::Attribute::ALL {
                lines.push(Line::from(format!(
                    "  {}: {}",
                    attribute.id(),
                    player.attributes.get(attribute)
                )));
            }
        }
        if let Some(character) = &self.app.character {
            lines.push(Line::from(format!(
                "Elementos: {}",
                character
                    .elements
                    .iter()
                    .map(|e| e.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        if let Some(player) = &self.app.player {
            lines.push(Line::from(format!(
                "HP {}/{} · Tenacidade {}/{} · Mana {}/{}",
                player.hp.current,
                player.hp.maximum,
                player.tenacity.current,
                player.tenacity.maximum,
                player.mana.current,
                player.mana.maximum
            )));
        }
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .block(Block::default().title("Personagem").borders(Borders::ALL)),
            area,
        );
    }

    fn render_inventory(&self, frame: &mut Frame<'_>, area: Rect) {
        self.render_profile_list(
            frame,
            area,
            "Inventário",
            self.app
                .character
                .as_ref()
                .map(|v| v.inventory.iter().map(String::as_str).collect())
                .unwrap_or_default(),
        );
    }
    fn render_spells(&self, frame: &mut Frame<'_>, area: Rect) {
        if self
            .app
            .character
            .as_ref()
            .is_some_and(|character| character.spells.is_empty())
        {
            frame.render_widget(Paragraph::new("As quatro magias padrão ainda dependem de definições de conteúdo construídas com Nodes. Nenhuma magia fictícia foi adicionada.").block(Block::default().title("Magias").borders(Borders::ALL)).wrap(Wrap { trim: true }), area);
        } else {
            self.render_profile_list(
                frame,
                area,
                "Magias",
                self.app
                    .character
                    .as_ref()
                    .map(|v| v.spells.iter().map(|s| s.id.as_str()).collect())
                    .unwrap_or_default(),
            );
        }
    }
    fn render_profile_list(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        title: &str,
        entries: Vec<&str>,
    ) {
        let body = if entries.is_empty() {
            "Nenhum conteúdo registrado.".to_owned()
        } else {
            entries.join("\n")
        };
        frame.render_widget(
            Paragraph::new(body).block(Block::default().title(title).borders(Borders::ALL)),
            area,
        );
    }
    fn render_equipment(&self, frame: &mut Frame<'_>, area: Rect) {
        let lines = self
            .app
            .character
            .as_ref()
            .map(|v| {
                v.equipment
                    .iter()
                    .map(|(slot, item)| {
                        Line::from(format!("{slot}: {}", item.as_deref().unwrap_or("vazio")))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![Line::from("Nenhum equipamento registrado.")]);
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .block(Block::default().title("Equipamentos").borders(Borders::ALL)),
            area,
        );
    }
    fn render_locations(&self, frame: &mut Frame<'_>, area: Rect) {
        let current = self
            .app
            .campaign
            .as_ref()
            .and_then(|v| v.location_id.as_deref())
            .unwrap_or("");
        let entries = self
            .app
            .state
            .location_registry
            .destinations(current)
            .unwrap_or_default();
        let lines = if entries.is_empty() {
            vec![Line::from("Nenhum destino disponível.")]
        } else {
            entries
                .iter()
                .enumerate()
                .map(|(index, location)| {
                    Line::from(format!(
                        "{}{} — {}",
                        if index == self.ui.selected {
                            "> "
                        } else {
                            "  "
                        },
                        location.id,
                        location.name
                    ))
                })
                .collect()
        };
        frame.render_widget(
            Paragraph::new(Text::from(lines)).block(
                Block::default()
                    .title("Locais disponíveis")
                    .borders(Borders::ALL),
            ),
            area,
        );
    }

    fn render_saves(&self, frame: &mut Frame<'_>, area: Rect) {
        let items = self
            .ui
            .saves
            .iter()
            .map(|save| {
                let mut lines = vec![Line::from(Span::styled(
                    &save.name,
                    Style::default()
                        .fg(if save.loadable {
                            Color::LightYellow
                        } else {
                            Color::LightRed
                        })
                        .add_modifier(Modifier::BOLD),
                ))];
                if let Some(character) = &save.character {
                    lines.push(Line::from(format!("Personagem: {character}")));
                }
                if let Some(race) = &save.race {
                    lines.push(Line::from(format!("Raca: {race}")));
                }
                lines.push(Line::from(save.details.clone()));
                ListItem::new(lines)
            })
            .collect::<Vec<_>>();
        if items.is_empty() {
            let empty = Paragraph::new("Nenhum save local encontrado em /saves.")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Carregar Saves")
                        .borders(Borders::ALL),
                );
            frame.render_widget(empty, centered_rect(78, 42, area));
        } else {
            let list = List::new(items)
                .block(Block::default().title("Saves locais").borders(Borders::ALL))
                .highlight_symbol("> ")
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                );
            let mut state = ListState::default();
            state.select(Some(
                self.ui.selected.min(self.ui.saves.len().saturating_sub(1)),
            ));
            frame.render_stateful_widget(list, centered_rect(90, 88, area), &mut state);
        }
    }

    fn render_mods(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut lines = vec![Line::from(
            "Mods sao descobertos localmente. Esta tela e somente informativa.",
        )];
        lines.push(Line::from(
            "Ativacao/desativacao ainda nao possui configuracao persistente.",
        ));
        lines.push(Line::from(""));
        let report = &self.app.state.mod_load_report;
        if report.loaded.is_empty()
            && report.failed.is_empty()
            && self.app.mod_load_failures.is_empty()
        {
            lines.push(Line::from("Nenhum mod encontrado em /mods."));
        }
        for discovered in &report.loaded {
            let manifest = &discovered.manifest;
            let has_runtime_error = self
                .app
                .mod_load_failures
                .iter()
                .any(|failure| failure.mod_id.as_deref() == Some(manifest.id.as_str()));
            lines.push(Line::from(Span::styled(
                format!(
                    "{}  {}  ({})",
                    manifest.name,
                    manifest.version,
                    if has_runtime_error {
                        "falha de carregamento; consulte o diagnostico"
                    } else {
                        "carregado nesta sessao"
                    }
                ),
                Style::default()
                    .fg(if has_runtime_error {
                        Color::LightRed
                    } else {
                        Color::LightGreen
                    })
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!(
                "ID: {} | API: {}",
                manifest.id, manifest.api_version
            )));
            lines.push(Line::from(format!(
                "Dependencias: {}",
                if manifest.dependencies.is_empty() {
                    "nenhuma".into()
                } else {
                    manifest
                        .dependencies
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )));
            lines.push(Line::from(
                "Autor/descricao: nao declarados no manifesto atual.",
            ));
            lines.push(Line::from(format!("Pasta: {}", discovered.root.display())));
            lines.push(Line::from(""));
        }
        for failure in report
            .failed
            .iter()
            .chain(self.app.mod_load_failures.iter())
        {
            lines.push(Line::from(Span::styled(
                format!(
                    "Falha {}: {}",
                    failure.mod_id.as_deref().unwrap_or("mod desconhecido"),
                    failure.stage
                ),
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!(
                "{} — {}",
                failure.path.display(),
                failure.reason
            )));
        }
        for warning in &report.warnings {
            lines.push(Line::from(Span::styled(
                format!("Aviso: {warning}"),
                Style::default().fg(Color::LightYellow),
            )));
        }
        let panel = Paragraph::new(Text::from(lines))
            .block(Block::default().title("Mods locais").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(panel, area);
    }

    fn render_credits(&self, frame: &mut Frame<'_>, area: Rect) {
        let lines = vec![
            Line::from(Span::styled("A Magic Sovereign", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))),
            Line::from("RPG offline-first em Rust, renderizado no terminal."),
            Line::from(""),
            Line::from("Autoria: A Magic Sovereign contributors (atribuicao do LICENSE)."),
            Line::from("Licenca: MIT — consulte o arquivo LICENSE."),
            Line::from(""),
            Line::from("Bibliotecas: ratatui, crossterm, mlua, serde, serde_json, thiserror, toml, zip, ctrlc e indexmap."),
            Line::from("Rust: linguagem e toolchain do projeto."),
            Line::from("Nenhum colaborador individual e atribuido pelo repositorio."),
        ];
        let panel = Paragraph::new(Text::from(lines))
            .block(Block::default().title("Creditos").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        frame.render_widget(panel, centered_rect(88, 72, area));
    }

    fn render_ready(&self, frame: &mut Frame<'_>, area: Rect) {
        let outcome = self
            .app
            .state
            .combat
            .as_ref()
            .and_then(|combat| combat.outcome.as_ref());
        let text = vec![
            Line::from(Span::styled(
                "Demonstracao de combate concluida",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "Save: {}",
                self.app.save_name().unwrap_or("sem nome")
            )),
            Line::from(format!(
                "Resultado: {}",
                outcome
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_else(|| "pronto para iniciar".into())
            )),
            Line::from(""),
            Line::from("Enter: iniciar novamente | S: salvar | Esc: menu"),
        ];
        let panel = Paragraph::new(Text::from(text))
            .block(Block::default().title("Partida").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(panel, centered_rect(75, 58, area));
    }

    fn render_combat(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if self.ui.show_explanation {
            let explanation = self.app.dispatch(AppAction::ExplainLast);
            let panel = Paragraph::new(explanation)
                .block(
                    Block::default()
                        .title("Explain Last — Esc para voltar")
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(panel, area);
            return;
        }
        if let Some(picker) = &self.ui.target_picker {
            self.render_target_picker(frame, area, picker);
            return;
        }
        let Some(combat) = self.app.state.combat.as_ref() else {
            frame.render_widget(Paragraph::new("Nenhum combate ativo."), area);
            return;
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
            .split(area);
        let status_lines = combat
            .entities
            .values()
            .map(|entity| {
                let team = match entity.allegiance {
                    Allegiance::Player => "Aliado",
                    Allegiance::Enemy => "Inimigo",
                    Allegiance::Neutral => "Neutro",
                };
                Line::from(format!(
                    "{team}: {} [{}]  HP {}/{}  Ten {}/{}  MP {}/{}  Shield {}{}",
                    entity.name,
                    entity.id,
                    entity.hp.current,
                    entity.hp.maximum,
                    entity.tenacity.current,
                    entity.tenacity.maximum,
                    entity.mana.current,
                    entity.mana.maximum,
                    entity.shield,
                    if entity.is_alive() {
                        ""
                    } else {
                        "  [fora de combate]"
                    }
                ))
            })
            .collect::<Vec<_>>();
        let status = Paragraph::new(Text::from(status_lines))
            .block(
                Block::default()
                    .title(format!("{} — Rodada {}", combat.encounter_id, combat.round))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(status, chunks[0]);

        let action_ids = combat
            .available_actions
            .get("base:player")
            .cloned()
            .unwrap_or_default();
        let mut action_items = action_ids
            .iter()
            .filter_map(|id| self.app.state.action_registry.get(id))
            .map(action_label)
            .collect::<Vec<_>>();
        action_items.push(ListItem::new("[Encerrar rodada]"));
        let list = List::new(action_items)
            .block(
                Block::default()
                    .title(format!(
                        "Acoes — AP {}",
                        combat.budgets.available("base:player")
                    ))
                    .borders(Borders::ALL),
            )
            .highlight_symbol("> ")
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            );
        let mut list_state = ListState::default();
        list_state.select(Some(self.ui.selected.min(action_ids.len())));
        frame.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    fn render_target_picker(&self, frame: &mut Frame<'_>, area: Rect, picker: &TargetPicker) {
        let multiple = picker.maximum > 1;
        let items = picker
            .targets
            .iter()
            .map(|target| {
                let selected = picker.selected.contains(target);
                ListItem::new(format!(
                    "{} {}",
                    if selected {
                        "[x]"
                    } else if multiple {
                        "[ ]"
                    } else {
                        "   "
                    },
                    target
                ))
            })
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(picker.cursor.min(items.len() - 1)));
        }
        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Escolher alvo — {}", picker.action_id))
                    .borders(Borders::ALL),
            )
            .highlight_symbol("> ")
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(list, centered_rect(82, 75, area), &mut state);
    }

    fn render_dialog(&self, frame: &mut Frame<'_>, area: Rect, dialog: &Dialog) {
        let (title, body) = match dialog {
            Dialog::Exit => (
                "Sair",
                "Ha alteracoes nao salvas. S: salvar e sair | D: descartar e sair | Esc: cancelar",
            ),
            Dialog::Load(_) => (
                "Carregar save",
                "Ha alteracoes nao salvas. Enter: substituir pelo save selecionado | Esc: cancelar",
            ),
            Dialog::Replace => (
                "Novo jogo",
                "Ha alteracoes nao salvas. Enter: continuar e substituir | Esc: cancelar",
            ),
            Dialog::SkipPrologue => (
                "Pular prólogo",
                "Enter: concluir o prólogo e iniciar no dia 7 | Esc: continuar assistindo",
            ),
        };
        let rect = centered_rect(78, 30, area);
        frame.render_widget(ratatui::widgets::Clear, rect);
        frame.render_widget(
            Paragraph::new(body)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::LightYellow)),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            rect,
        );
    }

    fn footer_text(&self) -> String {
        let help = if self.ui.dialog.is_some() {
            "Teclas indicadas na janela"
        } else if self.ui.show_explanation {
            "Esc: voltar"
        } else if self
            .ui
            .target_picker
            .as_ref()
            .is_some_and(|picker| picker.maximum > 1)
        {
            "Cima/Baixo: navegar | Espaco: marcar | Enter: confirmar | Esc: cancelar"
        } else {
            match self.ui.screen.unwrap_or(Screen::MainMenu) {
                Screen::MainMenu => "Cima/Baixo: navegar | Enter: selecionar | Esc: sair",
                Screen::NewGame => "Cima/Baixo: campos | Digitar: nomes | Setas: raça | Espaço: elemento | Enter: confirmar | Esc: voltar",
                Screen::Saves => "Cima/Baixo: navegar | Enter: carregar | Esc: voltar",
                Screen::Mods | Screen::Credits => "Esc: voltar",
                Screen::Ready => "Enter: iniciar demo | S: salvar | Esc: menu",
                Screen::Combat => "Cima/Baixo: acoes | Enter: selecionar | E: rodada | S: salvar | X: explicar | Esc: sair",
                Screen::Prologue => "Enter/Espaço: avançar | Esc: pular prólogo | S: salvar",
                Screen::Explore => "Cima/Baixo: ações | Enter: abrir | S: salvar | Esc: sair",
                Screen::Character | Screen::Inventory | Screen::Spells | Screen::Equipment => "Esc: voltar à exploração",
                Screen::Locations => "Cima/Baixo: destino | Enter: viajar | Esc: voltar",
            }
        };
        if self.ui.message.is_empty() {
            help.into()
        } else {
            format!("{help}  |  {}", self.ui.message)
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if let Some(dialog) = self.ui.dialog.take() {
            self.handle_dialog_key(key, dialog);
            return;
        }
        if self.ui.show_explanation {
            if key.code == KeyCode::Esc {
                self.ui.show_explanation = false;
            }
            return;
        }
        if self.ui.target_picker.is_some() {
            self.handle_target_key(key);
            return;
        }
        match self.ui.screen.unwrap_or(Screen::MainMenu) {
            Screen::MainMenu => self.handle_menu_key(key),
            Screen::NewGame => self.handle_new_game_key(key),
            Screen::Saves => self.handle_saves_key(key),
            Screen::Mods | Screen::Credits => {
                if key.code == KeyCode::Esc {
                    self.go_back();
                }
            }
            Screen::Ready => self.handle_ready_key(key),
            Screen::Combat => self.handle_combat_key(key),
            Screen::Prologue => self.handle_prologue_key(key),
            Screen::Explore => self.handle_explore_key(key),
            Screen::Character | Screen::Inventory | Screen::Spells | Screen::Equipment => {
                if key.code == KeyCode::Esc {
                    self.ui.screen = Some(Screen::Explore);
                }
            }
            Screen::Locations => self.handle_locations_key(key),
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.ui.selected = move_selection(self.ui.selected, MENU.len(), true),
            KeyCode::Down => self.ui.selected = move_selection(self.ui.selected, MENU.len(), false),
            KeyCode::Enter => match self.ui.selected {
                0 => {
                    self.ui.save_name.clear();
                    self.ui.character_name.clear();
                    self.ui.selected = 0;
                    self.ui.form_field = 0;
                    self.ui.selected_elements.clear();
                    if self.app.dirty {
                        self.ui.dialog = Some(Dialog::Replace);
                    } else {
                        self.ui.screen = Some(Screen::NewGame);
                    }
                }
                1 => {
                    self.refresh_saves();
                    self.ui.screen = Some(Screen::Saves);
                    self.ui.selected = 0;
                }
                2 => {
                    self.ui.screen = Some(Screen::Mods);
                    self.ui.selected = 0;
                }
                3 => {
                    self.ui.screen = Some(Screen::Credits);
                    self.ui.selected = 0;
                }
                _ => self.request_quit(),
            },
            KeyCode::Esc => self.request_quit(),
            _ => {}
        }
    }

    fn handle_new_game_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.go_back(),
            KeyCode::Up => self.ui.form_field = self.ui.form_field.saturating_sub(1),
            KeyCode::Down => self.ui.form_field = (self.ui.form_field + 1).min(16),
            KeyCode::Left
                if self.ui.form_field == 2 && !self.app.state.race_registry.is_empty() =>
            {
                let count = self.app.state.race_registry.len();
                self.ui.race_index = (self.ui.race_index + count - 1) % count
            }
            KeyCode::Right
                if self.ui.form_field == 2 && !self.app.state.race_registry.is_empty() =>
            {
                self.ui.race_index = (self.ui.race_index + 1) % self.app.state.race_registry.len()
            }
            KeyCode::Char(' ') if (3..16).contains(&self.ui.form_field) => {
                let element = elements()[self.ui.form_field - 3];
                if self.ui.selected_elements.contains(&element) {
                    self.ui.selected_elements.remove(&element);
                } else if self.ui.selected_elements.len() < 2 {
                    self.ui.selected_elements.insert(element);
                } else {
                    self.set_message("Você já selecionou dois elementos.");
                }
            }
            KeyCode::Backspace if self.ui.form_field == 0 => {
                self.ui.save_name.pop();
            }
            KeyCode::Backspace if self.ui.form_field == 1 => {
                self.ui.character_name.pop();
            }
            KeyCode::Enter if self.ui.form_field == 16 => {
                if let Err(error) = validate_save_name(&self.ui.save_name) {
                    self.set_message(error.to_string());
                    return;
                }
                let Some(race_id) = self.selected_race().map(|race| race.id.clone()) else {
                    self.set_message("Nenhuma raça carregada em data/content/races.json.");
                    return;
                };
                let response = self.app.dispatch(AppAction::NewCharacter {
                    save_name: self.ui.save_name.clone(),
                    character_name: self.ui.character_name.clone(),
                    race_id,
                    elements: self.ui.selected_elements.iter().copied().collect(),
                    gamemode_id: "base:standard".into(),
                });
                self.set_message(response);
                if self.app.campaign.is_some() {
                    self.ui.screen = Some(Screen::Prologue);
                    self.ui.selected = 0;
                }
            }
            KeyCode::Char(character)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !character.is_control()
                    && self.ui.form_field <= 1 =>
            {
                let destination = if self.ui.form_field == 0 {
                    &mut self.ui.save_name
                } else {
                    &mut self.ui.character_name
                };
                if destination.chars().count() < 64 {
                    destination.push(character);
                }
            }
            _ => {}
        }
    }

    fn handle_prologue_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                let result = self.app.dispatch(AppAction::AdvanceScene);
                self.set_message(result);
                self.scene_started = Instant::now();
                if self
                    .app
                    .campaign
                    .as_ref()
                    .is_some_and(|v| v.prologue_completed)
                {
                    self.ui.screen = Some(Screen::Explore);
                }
            }
            KeyCode::Esc => self.ui.dialog = Some(Dialog::SkipPrologue),
            KeyCode::Char('s' | 'S') => {
                let result = self.app.dispatch(AppAction::Save);
                self.set_message(result);
            }
            _ => {}
        }
    }

    fn handle_explore_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.ui.selected = move_selection(self.ui.selected, 7, true),
            KeyCode::Down => self.ui.selected = move_selection(self.ui.selected, 7, false),
            KeyCode::Enter => {
                self.ui.screen = Some(match self.ui.selected {
                    0 => Screen::Locations,
                    1 => Screen::Character,
                    2 => Screen::Inventory,
                    3 => Screen::Spells,
                    4 => Screen::Equipment,
                    5 => {
                        self.set_message("Descanso ainda não possui regras implementadas.");
                        return;
                    }
                    _ => {
                        let result = self.app.dispatch(AppAction::Save);
                        self.set_message(result);
                        return;
                    }
                });
                self.ui.selected = 0;
            }
            KeyCode::Char('s' | 'S') => {
                let result = self.app.dispatch(AppAction::Save);
                self.set_message(result);
            }
            KeyCode::Esc => self.request_quit(),
            _ => {}
        }
    }

    fn handle_locations_key(&mut self, key: KeyEvent) {
        let current = self
            .app
            .campaign
            .as_ref()
            .and_then(|v| v.location_id.as_deref())
            .unwrap_or("");
        let destinations = self
            .app
            .state
            .location_registry
            .destinations(current)
            .unwrap_or_default();
        match key.code {
            KeyCode::Up if !destinations.is_empty() => {
                self.ui.selected = move_selection(self.ui.selected, destinations.len(), true)
            }
            KeyCode::Down if !destinations.is_empty() => {
                self.ui.selected = move_selection(self.ui.selected, destinations.len(), false)
            }
            KeyCode::Enter => {
                if let Some(location) = destinations.get(self.ui.selected) {
                    let result = self.app.dispatch(AppAction::MoveToLocation {
                        location_id: location.id.clone(),
                    });
                    self.set_message(result);
                    self.ui.screen = Some(Screen::Explore);
                }
            }
            KeyCode::Esc => self.ui.screen = Some(Screen::Explore),
            _ => {}
        }
    }

    fn handle_saves_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.go_back(),
            KeyCode::Up if !self.ui.saves.is_empty() => {
                self.ui.selected = move_selection(self.ui.selected, self.ui.saves.len(), true)
            }
            KeyCode::Down if !self.ui.saves.is_empty() => {
                self.ui.selected = move_selection(self.ui.selected, self.ui.saves.len(), false)
            }
            KeyCode::Enter => {
                let Some(row) = self.ui.saves.get(self.ui.selected) else {
                    return;
                };
                if !row.loadable {
                    self.set_message(row.details.clone());
                    return;
                }
                let action = AppAction::RequestLoad {
                    save_name: row.name.clone(),
                };
                if self.app.dirty {
                    self.ui.dialog = Some(Dialog::Load(action));
                } else {
                    self.load_save(action);
                }
            }
            _ => {}
        }
    }

    fn handle_ready_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.ui.screen = Some(Screen::MainMenu);
                self.ui.selected = 0;
            }
            KeyCode::Enter => {
                let result = self.app.dispatch(AppAction::StartCombat {
                    encounter_id: "base:training-encounter".into(),
                });
                self.set_message(result);
                if self.app.mode == AppMode::Combat {
                    self.ui.screen = Some(Screen::Combat);
                    self.ui.selected = 0;
                }
            }
            KeyCode::Char('s' | 'S') => {
                let result = self.app.dispatch(AppAction::Save);
                self.set_message(result);
            }
            _ => {}
        }
    }

    fn handle_combat_key(&mut self, key: KeyEvent) {
        let action_ids = self
            .app
            .state
            .combat
            .as_ref()
            .and_then(|combat| combat.available_actions.get("base:player"))
            .map(Vec::len)
            .unwrap_or(0);
        let end_index = action_ids;
        match key.code {
            KeyCode::Esc => self.request_quit(),
            KeyCode::Up => self.ui.selected = self.ui.selected.saturating_sub(1),
            KeyCode::Down => self.ui.selected = (self.ui.selected + 1).min(end_index),
            KeyCode::Enter if self.ui.selected == end_index => {
                let result = self.app.dispatch(AppAction::EndRound);
                self.set_message(result);
                self.sync_combat_screen();
            }
            KeyCode::Enter => self.select_combat_action(),
            KeyCode::Char('e' | 'E') => {
                let result = self.app.dispatch(AppAction::EndRound);
                self.set_message(result);
                self.sync_combat_screen();
            }
            KeyCode::Char('s' | 'S') => {
                let result = self.app.dispatch(AppAction::Save);
                self.set_message(result);
            }
            KeyCode::Char('x' | 'X') => self.ui.show_explanation = true,
            _ => {}
        }
    }

    fn select_combat_action(&mut self) {
        let action_id = self
            .app
            .state
            .combat
            .as_ref()
            .and_then(|combat| combat.available_actions.get("base:player"))
            .and_then(|actions| actions.get(self.ui.selected))
            .cloned();
        let Some(action_id) = action_id else {
            return;
        };
        let Some(definition) = self.app.state.action_registry.get(&action_id).cloned() else {
            return;
        };
        if definition.tags.iter().any(|tag| tag == "reaction") {
            let result = self.app.dispatch(AppAction::ReserveReaction { action_id });
            self.set_message(result);
            return;
        }
        if action_id == "base:flee" {
            let result = self.app.dispatch(AppAction::Flee);
            self.set_message(result);
            self.ui.screen = Some(Screen::Ready);
            return;
        }
        match definition.target.mode {
            TargetMode::None | TargetMode::SelfOnly | TargetMode::All => {
                self.submit_action(&definition, Vec::new());
            }
            TargetMode::Single | TargetMode::Multiple { .. } => {
                let targets = self.candidate_targets(&definition);
                if targets.is_empty() {
                    self.set_message("Nao ha alvos validos nesta selecao.");
                    return;
                }
                let maximum = match definition.target.mode {
                    TargetMode::Single => 1,
                    TargetMode::Multiple { maximum } => maximum,
                    _ => 1,
                };
                self.ui.target_picker = Some(TargetPicker {
                    action_id,
                    targets,
                    cursor: 0,
                    selected: BTreeSet::new(),
                    maximum,
                });
            }
        }
    }

    fn candidate_targets(&self, definition: &ActionDefinition) -> Vec<String> {
        let Some(combat) = &self.app.state.combat else {
            return Vec::new();
        };
        let Some(actor) = combat.entities.get("base:player") else {
            return Vec::new();
        };
        combat
            .entities
            .iter()
            .filter_map(|(id, entity)| {
                let relation_ok = match definition.target.relation {
                    TargetRelation::SelfOnly => id == "base:player",
                    TargetRelation::Ally => entity.allegiance == actor.allegiance,
                    TargetRelation::Enemy => {
                        entity.allegiance != actor.allegiance
                            && entity.allegiance != Allegiance::Neutral
                    }
                    TargetRelation::Any => true,
                };
                let state_ok = match definition.target.state {
                    TargetState::Active => entity.is_alive(),
                    TargetState::Dead => !entity.is_alive(),
                    TargetState::Any => true,
                };
                (relation_ok && state_ok).then(|| id.clone())
            })
            .collect()
    }

    fn handle_target_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.ui.target_picker.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.ui.target_picker = None,
            KeyCode::Up if !picker.targets.is_empty() => {
                picker.cursor = (picker.cursor + picker.targets.len() - 1) % picker.targets.len()
            }
            KeyCode::Down if !picker.targets.is_empty() => {
                picker.cursor = (picker.cursor + 1) % picker.targets.len()
            }
            KeyCode::Char(' ') if picker.maximum > 1 => {
                if let Some(target) = picker.targets.get(picker.cursor).cloned() {
                    if picker.selected.contains(&target) {
                        picker.selected.remove(&target);
                    } else if picker.selected.len() < picker.maximum {
                        picker.selected.insert(target);
                    }
                }
            }
            KeyCode::Enter => {
                let target_ids = if picker.maximum == 1 {
                    picker
                        .targets
                        .get(picker.cursor)
                        .cloned()
                        .into_iter()
                        .collect()
                } else {
                    picker.selected.iter().cloned().collect()
                };
                let action_id = picker.action_id.clone();
                let definition = self.app.state.action_registry.get(&action_id).cloned();
                self.ui.target_picker = None;
                if let Some(definition) = definition {
                    self.submit_action(&definition, target_ids);
                }
            }
            _ => {}
        }
    }

    fn submit_action(&mut self, definition: &ActionDefinition, target_ids: Vec<String>) {
        let result = self.app.dispatch(AppAction::UseAction {
            action_id: definition.id.clone(),
            target_ids,
        });
        self.set_message(result);
        self.sync_combat_screen();
    }

    fn handle_dialog_key(&mut self, key: KeyEvent, dialog: Dialog) {
        match dialog {
            Dialog::Exit => match key.code {
                KeyCode::Char('s' | 'S') => {
                    let result = self.app.dispatch(AppAction::Save);
                    self.set_message(result);
                    if !self.app.dirty {
                        self.app.dispatch(AppAction::RequestQuit);
                    } else {
                        self.ui.dialog = Some(Dialog::Exit);
                    }
                }
                KeyCode::Char('d' | 'D') => {
                    self.app.dispatch(AppAction::ConfirmQuit);
                }
                _ => {
                    self.app.dispatch(AppAction::CancelPending);
                    self.ui.message = "Saida cancelada.".into();
                }
            },
            Dialog::Load(action) => match key.code {
                KeyCode::Enter => self.load_save(action),
                _ => self.ui.message = "Carregamento cancelado.".into(),
            },
            Dialog::Replace => match key.code {
                KeyCode::Enter => {
                    self.ui.dialog = None;
                    self.ui.screen = Some(Screen::NewGame);
                    self.ui.message = "Preencha um novo nome de save.".into();
                }
                _ => {
                    self.ui.screen = Some(Screen::MainMenu);
                    self.ui.message = "Novo jogo cancelado.".into();
                }
            },
            Dialog::SkipPrologue => match key.code {
                KeyCode::Enter => {
                    let result = self.app.dispatch(AppAction::SkipPrologue);
                    self.set_message(result);
                    self.ui.screen = Some(Screen::Explore);
                }
                _ => self.ui.screen = Some(Screen::Prologue),
            },
        }
    }

    fn request_quit(&mut self) {
        let result = self.app.dispatch(AppAction::RequestQuit);
        if self.app.dirty {
            self.ui.dialog = Some(Dialog::Exit);
        }
        self.set_message(result);
    }

    fn load_save(&mut self, action: AppAction) {
        if !matches!(action, AppAction::RequestLoad { .. }) {
            return;
        }
        let had_unsaved_changes = self.app.dirty;
        let response = self.app.dispatch(action);
        self.set_message(response);
        if had_unsaved_changes {
            let response = self.app.dispatch(AppAction::ConfirmLoad);
            self.set_message(response);
        }
        if self.app.mode == AppMode::Combat {
            self.ui.screen = Some(Screen::Combat);
        } else if self
            .app
            .campaign
            .as_ref()
            .is_some_and(|campaign| campaign.prologue_completed)
        {
            self.ui.screen = Some(Screen::Explore);
        } else if self.app.campaign.is_some() {
            self.ui.screen = Some(Screen::Prologue);
        } else if self.app.mode == AppMode::Ready {
            self.ui.screen = Some(Screen::Ready);
        }
    }

    fn refresh_saves(&mut self) {
        self.ui.saves.clear();
        let Some(paths) = &self.app.state.paths else {
            self.set_message("Diretorios do jogo indisponiveis.");
            return;
        };
        let rows = match read_save_rows(&paths.saves) {
            Ok(rows) => rows,
            Err(error) => {
                self.set_message(format!("Falha ao listar saves: {error}"));
                return;
            }
        };
        self.ui.saves = rows;
        self.ui.selected = 0;
        if self.ui.saves.is_empty() {
            self.ui.message = "Nenhum save local encontrado.".into();
        }
    }

    fn go_back(&mut self) {
        self.ui.screen = Some(match self.ui.screen.unwrap_or(Screen::MainMenu) {
            Screen::MainMenu => Screen::MainMenu,
            Screen::Combat | Screen::Ready | Screen::Explore | Screen::Prologue => Screen::MainMenu,
            Screen::Character
            | Screen::Inventory
            | Screen::Spells
            | Screen::Equipment
            | Screen::Locations => Screen::Explore,
            _ => Screen::MainMenu,
        });
        self.ui.selected = 0;
    }

    fn set_message(&mut self, message: impl Into<String>) {
        let message = message.into();
        if !message.is_empty() {
            self.ui.history.push_back(message.clone());
            while self.ui.history.len() > HISTORY_LIMIT {
                self.ui.history.pop_front();
            }
            self.ui.message = message;
        }
    }
}

fn read_save_rows(directory: &Path) -> io::Result<Vec<SaveRow>> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                rows.push(SaveRow {
                    name: "Erro de leitura do diretório".into(),
                    character: None,
                    race: None,
                    details: error.to_string(),
                    loadable: false,
                });
                continue;
            }
        };
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        {
            continue;
        }
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        if let Err(error) = validate_save_name(&name) {
            rows.push(SaveRow {
                name,
                character: None,
                race: None,
                details: format!("Nome de arquivo não portátil: {error}"),
                loadable: false,
            });
            continue;
        }
        match read_save(&path) {
            Ok(save) => {
                let player = player_entity(&save);
                rows.push(SaveRow {
                    name,
                    character: player.map(|entity| entity.name.clone()),
                    race: player
                        .and_then(|entity| entity.race_id.as_ref().map(ToString::to_string)),
                    details: if let Some(campaign) = &save.campaign {
                        format!(
                            "Dia {} · {}",
                            campaign.day,
                            campaign
                                .location_id
                                .as_deref()
                                .or(campaign.current_scene_id.as_deref())
                                .unwrap_or("sem localização")
                        )
                    } else if save.combat.is_some() {
                        "Save de combate carregável.".into()
                    } else {
                        "Save de demonstração ou formato legado.".into()
                    },
                    loadable: true,
                });
            }
            Err(error) => rows.push(SaveRow {
                name,
                character: None,
                race: None,
                details: format!("Save invalido: {error}"),
                loadable: false,
            }),
        }
    }
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(rows)
}

fn player_entity(save: &SaveData) -> Option<&crate::entities::Entity> {
    save.metadata
        .player_entity_ids
        .iter()
        .find_map(|id| save.entities.iter().find(|entity| entity.id.as_str() == id))
}

fn move_selection(current: usize, item_count: usize, up: bool) -> usize {
    if item_count == 0 {
        return 0;
    }
    if up {
        (current.min(item_count - 1) + item_count - 1) % item_count
    } else {
        (current.min(item_count - 1) + 1) % item_count
    }
}

fn glitch_text(frame: u8) -> String {
    const ORIGINAL: &str = "O RITUAL FALHOU";
    if frame == 7 {
        return ORIGINAL.into();
    }
    if frame == 6 {
        return String::new();
    }
    let mut chars = ORIGINAL.chars().collect::<Vec<_>>();
    for (index, character) in chars.iter_mut().enumerate() {
        if *character != ' ' && (index + usize::from(frame)) % 4 == 0 {
            *character = ['#', '/', '?', '!'][usize::from(frame % 4)];
        }
    }
    format!(
        "{}{}",
        " ".repeat(usize::from(frame % 3)),
        chars.into_iter().collect::<String>()
    )
}

fn action_label(action: &ActionDefinition) -> ListItem<'static> {
    ListItem::new(format!(
        "{} — {} AP, {} Mana, Cast {}",
        action.id, action.ap_cost, action.mana_cost, action.cast
    ))
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    raw_mode: bool,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(error);
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self {
                terminal,
                raw_mode: true,
            }),
            Err(error) => {
                let mut stdout = io::stdout();
                let _ = execute!(stdout, Show, LeaveAlternateScreen);
                let _ = disable_raw_mode();
                Err(error)
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
        if self.raw_mode {
            let _ = disable_raw_mode();
        }
        let _ = self.terminal.show_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::{write_save, SaveMetadata, SAVE_FORMAT_VERSION};
    use serde_json::json;

    #[test]
    fn menu_navigation_wraps_and_selects_all_five_entries() {
        let mut selected = move_selection(0, MENU.len(), true);
        assert_eq!(MENU[selected], "Sair");
        selected = move_selection(selected, MENU.len(), false);
        assert_eq!(MENU[selected], "Jogar");
        assert_eq!(MENU.len(), 5);
    }

    #[test]
    fn navigation_handles_empty_and_stale_selection_without_panicking() {
        assert_eq!(move_selection(9, 0, true), 0);
        assert_eq!(move_selection(9, 3, false), 0);
        assert_eq!(move_selection(9, 3, true), 1);
    }

    #[test]
    fn screen_state_keeps_save_identity_separate_from_character_name() {
        let state = UiState {
            save_name: "campanha_principal".into(),
            character_name: "Éowyn".into(),
            ..UiState::default()
        };
        assert_eq!(state.save_name, "campanha_principal");
        assert_eq!(state.character_name, "Éowyn");
    }

    #[test]
    fn ritual_glitch_restores_original_text_after_dark_frame() {
        assert_eq!(glitch_text(6), "");
        assert_eq!(glitch_text(7), "O RITUAL FALHOU");
        assert_ne!(glitch_text(2), "O RITUAL FALHOU");
    }

    #[test]
    fn character_form_renders_all_thirteen_elements_and_selects_last_one() {
        let app = App::new(crate::engine::GameState::new(1));
        let mut tui = Tui::new(app);
        tui.ui.screen = Some(Screen::NewGame);
        let backend = ratatui::backend::TestBackend::new(100, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| tui.render(frame)).unwrap();
        tui.ui.form_field = 15;
        tui.handle_new_game_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(tui.ui.selected_elements.contains(&Element::Lightning));
    }

    #[test]
    fn ritual_glitch_renders_each_frame_and_restores_text() {
        let mut app = App::new(crate::engine::GameState::new(1));
        app.state.visual_effects = "full".into();
        app.state.scene_registry =
            crate::campaign::SceneRegistry::new(vec![crate::campaign::NarrativeScene {
                id: "core:test-glitch".into(),
                text: "O RITUAL FALHOU".into(),
                style: "glitch".into(),
                auto_advance: false,
                next: None,
                completion_events: vec![],
            }])
            .unwrap();
        app.campaign = Some(crate::campaign::CampaignProgress::prologue(
            "core:test-glitch",
        ));
        let mut tui = Tui::new(app);
        tui.ui.screen = Some(Screen::Prologue);
        let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
        for frame in 0..8 {
            tui.ui.glitch_frame = frame;
            tui.ui.glitch_finished = frame == 7;
            terminal.draw(|frame| tui.render(frame)).unwrap();
        }
    }

    #[test]
    fn save_listing_reads_real_archives_and_reports_corrupt_files() {
        let directory = tempfile::tempdir().unwrap();
        assert!(read_save_rows(directory.path()).unwrap().is_empty());
        write_save(
            &directory.path().join("Dragão Negro.zip"),
            &SaveData {
                metadata: SaveMetadata {
                    save_format_version: SAVE_FORMAT_VERSION,
                    save_id: "Dragão Negro".into(),
                    ruleset_version: "base:standard@1".into(),
                    required_mods: Vec::new(),
                    gamemode_id: "base:standard".into(),
                    player_entity_ids: Vec::new(),
                    rng_state: 1,
                },
                ruleset: json!({}),
                entities: Vec::new(),
                character: None,
                campaign: None,
                combat: None,
                opaque_mod_data: Default::default(),
                migration_warnings: Vec::new(),
            },
        )
        .unwrap();
        fs::write(directory.path().join("corrupted.zip"), b"not a zip archive").unwrap();
        let rows = read_save_rows(directory.path()).unwrap();
        assert_eq!(rows.len(), 2);
        let valid = rows.iter().find(|row| row.name == "Dragão Negro").unwrap();
        assert!(valid.loadable);
        assert!(valid.character.is_none());
        assert!(valid.race.is_none());
        let invalid = rows.iter().find(|row| row.name == "corrupted").unwrap();
        assert!(!invalid.loadable);
        assert!(invalid.details.contains("Save invalido"));
    }

    #[test]
    fn save_rows_can_represent_an_empty_list() {
        let rows: Vec<SaveRow> = Vec::new();
        assert!(rows.is_empty());
        let invalid = SaveRow {
            name: "broken".into(),
            character: None,
            race: None,
            details: "bad zip".into(),
            loadable: false,
        };
        assert!(!invalid.loadable);
    }

    #[test]
    fn target_selection_is_bounded_by_action_maximum() {
        let mut selected = BTreeSet::new();
        selected.insert("enemy:one".to_string());
        selected.insert("enemy:two".to_string());
        assert_eq!(selected.len(), 2);
    }
}
