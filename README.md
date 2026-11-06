DctrXD´s JEI é um port NÃO OFICIAL do Just Enough Items (por mezz) para o Horizon. Não é afiliado ao autor original, à Mojang ou à Microsoft.

# DctrXD´s JEI

[Português](#português) · [English](#english)

## Português

Visualizador de itens e receitas para o [Horizon](https://github.com/horizon-rust/horizon), reescrito em Rust a partir do
[Just Enough Items](https://github.com/mezz/JustEnoughItems) (licença MIT, por mezz).

**Situação: ainda não foi testado dentro do jogo.** Os testes automáticos passam, mas ninguém viu o mod rodando. Veja
[docs/PROGRESS.md](docs/PROGRESS.md).

### O que ele faz

- **Lista de itens** ao lado do inventário, dos baús e do criativo, com páginas.
- **Busca** como a do JEI: palavras, `"frases"`, `-excluir`, `a|b`, `@mod`, `#tag`, `&id`.
- **Receitas (R ou clique) e usos (U ou clique direito)**: fabricação, fornalha, defumador, alto-forno, fogueira, cortador de
  pedra, ferraria, poções e páginas de tags, com histórico.
- **Botão `+`**: preenche a grade com uma receita do seu livro. Shift preenche o máximo.
- **Favoritos (A)**: de itens e de receitas, à esquerda da tela. Arraste para reordenar.
- **Modo cheat** (Ctrl+K ou botão `C`), desligado por padrão: o clique dá itens no criativo. Em survival, só se você ligar
  "Cheat pode usar /give".
- **Modo edição** (Ctrl+J): o clique esconde itens da lista.
- **Atalhos**: Ctrl+O esconde a lista, Ctrl+F vai para a busca, Backspace volta, PgUp e PgDn mudam a página. As teclas ficam em
  Opções > Mods.
- **Categorias de outros mods** por dados, sem Rust: [examples/sample_category](examples/sample_category/README.md).

### O que ele não faz

Combustível, compostagem, bigorna, rebolo, fluidos e busca por cor, entre outros. A lista completa, com os motivos, está em
[docs/PORTING_NOTES.md](docs/PORTING_NOTES.md).

### Compilar

```powershell
horizon-launcher native-build mixins\dctrxds_jei --instance <sua-instância> --trust-native-code
```

**Atenção:** mods nativos do Horizon não rodam em sandbox e têm as permissões da sua conta. Este mod lê as tabelas do jogo e
grava somente em `config/dctrxds_jei/` e `config/dctrxds_jei-client.toml`. Ele só usa a rede pelo próprio jogo, quando você liga
o `/give` e pede um item. Leia o código antes de compilar.

Licença: [MIT](LICENSE). O que veio de outros projetos está em [NOTICE.md](NOTICE.md) e [LICENSES/](LICENSES/).

## English

DctrXD´s JEI is an UNOFFICIAL port of Just Enough Items (by mezz) to Horizon. It is not affiliated with the original author,
Mojang or Microsoft.

An item and recipe viewer for [Horizon](https://github.com/horizon-rust/horizon), rewritten in Rust from
[Just Enough Items](https://github.com/mezz/JustEnoughItems) (MIT licence, by mezz).

**Status: not tested inside the game yet.** The automated tests pass, but nobody has seen the mod running. See
[docs/PROGRESS.md](docs/PROGRESS.md).

### What it does

- **Item list** beside the inventory, chests and the creative screen, in pages.
- **Search** as in JEI: words, `"phrases"`, `-exclude`, `a|b`, `@mod`, `#tag`, `&id`.
- **Recipes (R or click) and uses (U or right click)**: crafting, furnace, smoker, blast furnace, campfire, stonecutter, smithing,
  brewing and tag pages, with history.
- **`+` button**: fills the grid with a recipe from your book. Shift fills as many as possible.
- **Bookmarks (A)**: of items and recipes, left of the screen. Drag to reorder.
- **Cheat mode** (Ctrl+K or the `C` button), off by default: a click gives items in creative. In survival, only if you turn on
  "Cheat may use /give".
- **Edit mode** (Ctrl+J): a click hides items from the list.
- **Keys**: Ctrl+O hides the list, Ctrl+F goes to search, Backspace goes back, PgUp and PgDn change page. Keys are in
  Options > Mods.
- **Categories from other mods** with data and no Rust: [examples/sample_category](examples/sample_category/README.md).

### What it doesn't do

Fuel, composting, anvil, grindstone, fluids and colour search, among others. The full list and the reasons are in
[docs/PORTING_NOTES.md](docs/PORTING_NOTES.md).

### Build

```powershell
horizon-launcher native-build mixins\dctrxds_jei --instance <your-instance> --trust-native-code
```

**Warning:** Horizon's native mods are not sandboxed and run with your account's permissions. This mod reads the game's tables
and writes only to `config/dctrxds_jei/` and `config/dctrxds_jei-client.toml`. It only uses the network through the game itself,
when you turn on `/give` and ask for an item. Read the source before building.

Licence: [MIT](LICENSE). What came from other projects is listed in [NOTICE.md](NOTICE.md) and [LICENSES/](LICENSES/).

NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
