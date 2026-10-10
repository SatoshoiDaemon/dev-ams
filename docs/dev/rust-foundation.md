# Fundação Rust

## Decisões técnicas

- Rust mínimo: `1.80`, edição `2021`.
- Terminal: `crossterm`, mantido atrás da camada `terminal`; o estado e o combate não dependem dele.
- Configuração: `toml`; conteúdo e saves: `serde_json`.
- Saves: `zip` com compressão Deflate; o conteúdo permanece JSON legível após extração.
- Lua: `mlua` com `lua54` e `vendored`, portanto o runtime é compilado dentro do executável.
- Erros: `thiserror` em fronteiras de dados, com caminho, arquivo e causa contextualizados.
- Números: inteiros para estado persistente e uma função central de `floor` para resultados fracionários. O RNG inicial é um XorShift determinístico com seed explícita.

## Organização dos módulos

`src/lib.rs` é a raiz da biblioteca e declara a API pública; `src/main.rs` é o
ponto de entrada do executável. As implementações dos módulos ficam em
diretórios `src/<módulo>/`, normalmente em `mod.rs`. Os caminhos públicos já
existentes, como `ams::combat` e `ams::saves`, são preservados pela declaração
dos módulos na raiz da biblioteca.

A migração para esses diretórios foi estrutural. Ela não dividiu internamente
`combat`, `engine`, `magic` ou outros módulos. Uma divisão futura deve seguir
responsabilidades reais, sem gerar arquivos, diretórios ou camadas artificiais.
CLI e TUI reutilizam a mesma aplicação e os mesmos sistemas de jogo. Estado,
regras e resultados devem permanecer independentes da apresentação; modding,
conteúdo data-driven, operação offline e propriedade local dos dados continuam
sendo requisitos arquiteturais.

## Inicialização

`ams::initialize(root)` carrega `config.toml`, resolve caminhos relativos à raiz da instalação, cria `saves/`, `mods/`, `data/`, `gamemodes/` e `logs/`, e abre `logs/latest.log`.

Conteúdo JSON é descoberto recursivamente, ordenado pelo caminho e inserido em um `BTreeMap` por ID estável no formato `namespace:name`. Isso torna a ordem independente do sistema de arquivos.

## Vertical slice implementado

A fundação não visual agora inclui:

- aritmética `i64` determinística com razões inteiras e basis points;
- atributos, recursos derivados, Power, limite de quatro fontes e Scaling;
- ações, AP, Cast, Priority, TargetSpec e ordenação determinística;
- Accuracy, Evasion, Parry, Defense, Block, Shield, Tenacity, HP e True Damage;
- eventos limitados por profundidade/quantidade e pacote substituível de `explain last`;
- registro e ciclo de vida data-driven de Status Effects em `data/content/statuses.json`;
- editor de spells e catálogo de Glyphs em `data/glyphs.json`;
- saves ZIP versionados com JSON e preservação de dados opacos de mods;
- descoberta determinística de mods e Lua 5.4 embutido sem IO, OS ou carregamento de módulos nativos;
- `CombatSession` serializável, rodadas fechadas, Cast, Block/Parry, Bonus Actions, encontro de treino e Demon demonstrativo;
- CLI textual, saves v2 durante combate, gamemodes, logs históricos e crash reports.

O loop de exploração e o conteúdo amplo do jogo continuam como camadas futuras. A API Lua v1 já expõe registro no carregamento, queries por cópia, callbacks determinísticos e mutações enfileiradas como `EngineCommand`. O limite exato está documentado em [Combat Vertical Slice Status](vertical-slice-status.md).
