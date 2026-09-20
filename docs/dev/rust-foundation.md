# Fundação Rust

## Decisões técnicas

- Rust mínimo: `1.80`, edição `2021`.
- Terminal: `crossterm`, mantido atrás da camada `terminal`; o estado e o combate não dependem dele.
- Configuração: `toml`; conteúdo e saves: `serde_json`.
- Saves: `zip` com compressão Deflate; o conteúdo permanece JSON legível após extração.
- Lua: `mlua` com `lua54` e `vendored`, portanto o runtime é compilado dentro do executável.
- Erros: `thiserror` em fronteiras de dados, com caminho, arquivo e causa contextualizados.
- Números: inteiros para estado persistente e uma função central de `floor` para resultados fracionários. O RNG inicial é um XorShift determinístico com seed explícita.

## Inicialização

`ams::initialize(root)` carrega `config.toml`, resolve caminhos relativos à raiz da instalação, cria `saves/`, `mods/`, `data/`, `gamemodes/` e `logs/`, e abre `logs/latest.log`.

Conteúdo JSON é descoberto recursivamente, ordenado pelo caminho e inserido em um `BTreeMap` por ID estável no formato `namespace:name`. Isso torna a ordem independente do sistema de arquivos.

## Limites atuais

Esta etapa implementa a fundação não visual. A aplicação ainda não possui loop de exploração, combate completo, migrações de saves ou uma API Lua de conteúdo. Esses pontos devem ser adicionados sobre os módulos existentes, sem mover regras para a apresentação.
