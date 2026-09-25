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
