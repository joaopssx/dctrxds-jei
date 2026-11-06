# Teste manual / Manual test

**Ninguém executou isto ainda.** Anote o resultado de cada número. O teste usa uma instância separada, então seus mundos atuais
não são tocados.

Leia o código antes: a flag abaixo declara que você confia nele com acesso total ao seu computador. A primeira compilação pode
levar de 20 a 30 minutos, segundo a documentação do Horizon. Anote o tempo.

```powershell
$L = "$env:LOCALAPPDATA\Programs\Horizon Launcher\horizon-launcher.exe"
& $L instance create dctrxds-test
& $L native-build "C:\Users\jpsen\Documents\code\rs\dctrxd-ports\dctrxds-jei\mixins\dctrxds_jei" --instance dctrxds-test --trust-native-code
& $L play --instance dctrxds-test --world Teste --creative
```

**Não copie o `.hnmod` para `horizon-mods\` nem o renomeie para `.hmod`.** Essa pasta é só de mods de conteúdo: o jogo não compila
nada dela e diz que falta o `horizon-mod.toml`. O mod só existe no jogo depois do `native-build` acima, e a instância passa a jogar
uma versão `native-...`, listada em `& $L versions --installed`.

Se a grade não aparecer, diminua a escala da interface ou aumente a janela. Para medir: em
`config\dctrxds_jei-client.toml` da instância, ponha `show_stats = true`; FPS e tempo de desenho aparecem sob o inventário.

## Base (Fase 0)

| # | Faça isto | Deve acontecer |
|---|---|---|
| 1 | Aperte E | A lista aparece à direita, com páginas e caixa de busca |
| 2 | Em Opções > Mods | Aparece `DctrXD´s JEI` com o `´` certo, sete teclas e os botões de configuração |
| 3 | Clique fora da lista, em um slot | O inventário funciona normalmente |
| 4 | Abra um baú e um forno | A lista também aparece |
| 5 | Com `show_stats`, aperte Ctrl+O e volte | A lista some e volta. Anote o FPS com e sem |

## Busca (Fase 1)

| # | Faça isto | Deve acontecer |
|---|---|---|
| 6 | Clique na caixa e digite `oak` | A lista filtra; a letra `e` não fecha o inventário |
| 7 | Digite `sword -iron` | Espadas, sem as de ferro |
| 8 | Digite `@minecraft #logs\|#planks` | Toras e tábuas |
| 9 | Clique direito na caixa | A busca limpa |
| 10 | Passe a roda sobre a lista; use `<` e `>` | A página muda |
| 11 | Mude a escala da interface de 1 a 4 | A lista nunca cobre o inventário e nada trava |
| 12 | Abra o inventário com efeitos de poção ativos | **Veja se os ícones se sobrepõem à lista** (limitação esperada) |
| 13 | Abra o livro de receitas | **Veja se a lista se sobrepõe a ele** |

## Receitas (Fase 2)

| # | Faça isto | Deve acontecer |
|---|---|---|
| 14 | Clique em Tábuas de carvalho | A página de receitas abre; a tora aparece |
| 15 | Passe o mouse em um slot do baú e aperte R | **Pergunta-chave: abre a receita daquele item?** |
| 16 | Clique direito em Graveto | Usos, com abas |
| 17 | Clique em um slot da receita, três vezes, depois Backspace três vezes | Volta passo a passo e por fim fecha a página |
| 18 | Esc com a página aberta | Fecha só a página, não o inventário |
| 19 | Abra cada aba: fornalha, defumador, alto-forno, fogueira, cortador, ferraria, poções, tags | Tudo aparece, sem travar |
| 20 | Entre em um servidor vanilla | A lista funciona, com as receitas do próprio jogo |

## Grade e cheat (Fase 4)

| # | Faça isto | Deve acontecer |
|---|---|---|
| 21 | Na mesa de trabalho, abra uma receita conhecida e clique `+` | A grade é preenchida; Shift preenche o máximo |
| 22 | Em uma receita desconhecida | `+` fica apagado e não faz nada |
| 23 | Ctrl+K e clique em um item, no criativo | Você ganha 1; Shift dá uma pilha |
| 24 | Em `/gamemode survival`, repita | Recusa no chat, sem travar |
| 25 | Ligue "Cheat pode usar /give" e repita | O jogo executa `/give`, se você tiver permissão |

## Favoritos e configurações (Fase 5)

| # | Faça isto | Deve acontecer |
|---|---|---|
| 26 | Aperte A sobre itens | Aparecem à esquerda; A de novo remove |
| 27 | Arraste um favorito sobre outro | A ordem muda |
| 28 | Clique `*` em uma receita | Ela entra nos favoritos e abre sozinha |
| 29 | Feche e abra o jogo | Favoritos continuam |
| 30 | Ctrl+J e clique em um item; Ctrl+J de novo | O item some da lista |
| 31 | Corrompa `config\dctrxds_jei\bookmarks.json` e reinicie | Sem travar; o arquivo vira `.bak` |
| 32 | Edite `client.toml` com o jogo aberto | Vale em até alguns segundos |
| 33 | Mude o idioma do jogo para português | A interface segue |

## Variantes e outros mods (Fases 6 e 7)

| # | Faça isto | Deve acontecer |
|---|---|---|
| 34 | Busque `potion` | Aparecem poções com nome, como `Potion (swiftness)`, com o ícone certo |
| 35 | Compile `examples\sample_category` com `cargo horizon-mod build`, ponha o `.hmod` em `horizon-mods\` da instância e ligue | Em usos de Pedregulho, aba "Crushing" |

**Para responder:** me diga o resultado de cada número, o tempo da compilação e o FPS.
