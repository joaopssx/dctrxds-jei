# Sample category

A content mod that adds a category to DctrXD´s JEI with data and no Rust.

- `data/<namespace>/dctrxds_jei/categories/<name>.json` declares the category.
- `recipe_type` names a recipe type outside `minecraft`; recipes of that type go in `data/<namespace>/recipe/`.
- `inputs` and `outputs` name the fields of that recipe's JSON. Inputs are read as ingredients: an item, a `#tag` or a list.
  Outputs are read as a result: `id` and `count`.

The game doesn't craft these recipes: this only shows them. Build it with `cargo horizon-mod build` and turn it on in
Options > Mods. Fields:

| Field | Required | Meaning |
|---|---|---|
| `title` | Yes | The tab's name |
| `icon` | Yes | The tab's item |
| `catalysts` | No | Items whose uses show the whole category |
| `recipe_type` | Yes | The recipes it shows |
| `inputs`, `outputs` | At least one field in one of them | Where to read each slot |

It works in single player and on a LAN host. Joining another server, the folder isn't read.
