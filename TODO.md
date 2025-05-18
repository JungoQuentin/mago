# ajoute la resolution de declaration/implementation (`goto_definition`)

Il y a plein de trucs cool a faire avec un LSP.

MAIS.

C'est tres large, tres vaste, tres complexe. Donc je prefere restreindre au maximum le truc.

Ca me permettra de soumettre un truc de base plus rapidement, et de gagner en certitude pour le reste.

Donc pour l'instant, on ne fait que le `goto_definition`.

ET

On scope a un seul "type de symbol", au choix, definition d'une :

- classe
- variable
- fonction
- methode

Pour un premier truc, le plus simple me semble evidemement la variable. Sauf que `mago find $variable` ne fonctionne pas.
Donc peut etre que la fonction serait plus simple ?
Je vais essayer de partir sur l'utilisation de l'AST direct, donc la variable c'est bien !

-> sauf qu'il a les symbol pour les fn (et pas les variables...)

## 1. la methode simple : avec le `find`

il existe une commande `mago find <query>` qui permet de trouver plusieurs type de reference (dont declaration, definition, implementation)

donc pas besoin de tout comprendre, juste call (ou copier) la fonction `find_references` et de récuperer ce qui m'interesse

et ensuite faire toute l'integration LSP (d'abord dans vscode izi, puis dans zed avec le plugin)

1. trouver le symbol sur lequel on a cliqué - (donc tout le string en entier)
2. le mettre dans le `find_references`

### 1. trouver le symbol

- le client du lsp me donne seulement la position du curseur quand l'action à été call
  - autrement dis, c'est a moi d'avoir la logic de deduire que c'est au milieu d'une class
  - je vais voir si la solution c'est tree-sitter -> non
  - du coup go voir vers mago si on a quelque chose pour ca

## 2. faire les choses bien

Bon je me suis aventure quelque part ou c'est plus complexe, mais aussi plus complet !

Le mieux c'est clairement de parser le fichier, puis de trouver l'indentifiant sous mon curseur.

Pour aller plus loin, il faudrait que j'ai le `sementics` de tout le projet, et que je trouve la
definition/declaration

- [ ] avoir tous les find du walker

# kb

- tree-sitter : un builder de parser (donc pas utile vu que `mago` a le sien)
- en PHP definition/declaration c'est la meme chose (pas de header comme en `C/C++`)
