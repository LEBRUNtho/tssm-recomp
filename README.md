# Bob l'éponge, le film sur iPhone et iPad

**Le jeu GameCube *The SpongeBob SquarePants Movie* (2004), jouable sur iPhone et iPad, sans émulateur.**
Version bêta.

Ce dépôt est un fork de [tssm-recomp](https://github.com/WR3CKINGBALL/tssm-recomp), de WR3CKINGBALL, qui a fait
tout le travail difficile : traduire le code du jeu en programme natif pour Windows et Android. Ce fork y ajoute
trois choses :

- la version **iPhone et iPad** ;
- la **version européenne** du jeu (français et néerlandais), en plus de la version américaine ;
- macOS comme machine de mise au point.

Le mode d'emploi d'origine (Windows, Android) est dans [`README-original.md`](README-original.md).

## Ce que ça donne

- Le jeu complet, en plein écran large, sans jailbreak ni JIT.
- Des commandes tactiles à l'écran (le bouton en forme d'œil les cache), ou une manette (MFi, PlayStation, Xbox,
  Switch Pro).
- La version européenne se lance dans la langue du téléphone, français ou néerlandais.
- Les vibrations du jeu, dans le téléphone ou dans la manette.
- Les textures plus fines de la version Xbox, si vous fournissez le pack (voir plus bas).
- Les sauvegardes, les réglages et le journal visibles dans l'app Fichiers, sous le nom de l'app.
- Le jeu se met en pause quand on quitte l'app et reprend au retour.

## Ce qu'il faut

- **Votre propre disque du jeu**, sous forme d'image `.iso` ou `.gcm` : rien du jeu n'est fourni ici.
  Deux versions sont acceptées : l'américaine (GGVE78, révision 1) et l'européenne français/néerlandais
  (GGVX78, révision 0). Une image dans un autre format (`.rvz`, `.ciso`...) doit d'abord être convertie en `.iso`,
  par exemple avec Dolphin ou [nod](https://github.com/encounter/nod).
- **Un Mac** avec Xcode et [Rust](https://rustup.rs/).
- Un outil de sideloading pour installer l'app (AltStore, Sideloadly, Feather...), ou votre propre certificat.

Il n'y a pas d'application toute faite à télécharger, et il ne peut pas y en avoir : le programme est fabriqué
à partir du code du jeu qui se trouve sur votre disque. L'app obtenue contient les fichiers de votre disque,
elle est donc pour vos appareils uniquement et ne doit pas être partagée.

## Construction

1. Installez Xcode et Rust, puis ajoutez la cible iPhone : `rustup target add aarch64-apple-ios`
2. Dans le dossier du dépôt : `tools/ios.sh chemin/vers/le/disque.iso ipa`
3. Installez `build/ios/SpongeBobMovie-<version>.ipa` avec votre outil de sideloading.

La première construction prend quelques minutes. En option, devant la commande :

- `PACK=chemin/vers/le/pack.zip` embarque un pack de textures au format Dolphin, par exemple
  [celui de la version Xbox](https://gamebanana.com/mods/648590). Les textes du jeu sont alors plus nets aussi.
- `ICON=image.png` remplace l'icône, qui est sinon le badge de Bob tiré du disque.
- `APP_NAME="..."` change le nom affiché sous l'icône.

Le début de [`tools/ios.sh`](tools/ios.sh) explique comment signer avec son propre certificat et comment
essayer au simulateur.

## Bêta : ce qu'il faut savoir

- La version européenne a été essayée sur un iPhone, au doigt. La version américaine n'a pour l'instant tourné
  que sur Mac et au simulateur d'iPhone.
- Les manettes et les vibrations sont écrites mais pas encore essayées sur un vrai appareil.
- Le jeu n'a pas été parcouru jusqu'au bout sur iPhone : des surprises restent possibles.
- La version européenne ouvre toujours sur son choix de langue, comme sur la console.

Un souci ? Ouvrez une *issue* en précisant la version du jeu, votre appareil et ce qui s'est passé. Le fichier
`log.txt`, dans le dossier de l'app (app Fichiers), aide beaucoup.

## Pour les curieux

- `crates/tssm/src/ios.rs` : ce qui est propre à l'iPhone (fichiers, manettes, vibrations, langue).
- `config/addresses.txt` : ce que le port connaît du code du jeu, par nom de symbole. Le traducteur cherche
  chaque nom dans la version du jeu qu'on lui donne, ce qui permet au même programme de se construire pour les
  deux disques.
- `tools/ios.sh` et `tools/ios/` : la construction de l'app.

## D'où ça vient

Petit projet perso : je m'amuse avec une IA (Claude) à faire tourner de vieux jeux sur iPhone et iPad, comme
[l'oncle Ernest](https://github.com/LEBRUNtho/oncle-ernest-ios) ou [Forestia](https://github.com/LEBRUNtho/forestia-ios).
Rien de professionnel, aucune prétention. Tout le mérite du portage d'origine revient à WR3CKINGBALL.

## Droits

Le jeu, ses personnages, images, musiques, voix et textes appartiennent à leurs ayants droit (Nickelodeon et
Viacom, THQ, Heavy Iron Studios, Nintendo). Ce projet n'a aucun lien avec eux, n'est pas officiel, ne fournit
aucun fichier ni aucun code du jeu et ne rapporte rien à personne.

Si vous êtes ayant droit et que quelque chose ici vous gêne, ouvrez une *issue* : le contenu concerné sera retiré
rapidement, sans discussion.

Le dépôt d'origine ne précise pas de licence : son code reste celui de son auteur. Les ajouts de ce fork lui sont
offerts, il peut en faire ce qu'il veut.
