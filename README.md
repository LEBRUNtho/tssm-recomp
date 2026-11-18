# Bob l'éponge, le film sur iPhone et iPad / The SpongeBob SquarePants Movie on iPhone and iPad

[Français](#français) | [English](#english)

## Français

**Le jeu GameCube *The SpongeBob SquarePants Movie* (2004), jouable sur iPhone et iPad, sans émulateur.**
Version bêta.

Ce dépôt est un fork de [tssm-recomp](https://github.com/WR3CKINGBALL/tssm-recomp), de WR3CKINGBALL, qui a fait
tout le travail difficile : traduire le code du jeu en programme natif pour Windows et Android. Ce fork y ajoute
trois choses :

- la version **iPhone et iPad** ;
- la **version européenne** du jeu (français et néerlandais), en plus de la version américaine ;
- macOS comme machine de mise au point.

Le mode d'emploi d'origine (Windows, Android) est dans [`README-original.md`](README-original.md).

### Ce que ça donne

- Le jeu complet, en plein écran large, sans jailbreak ni JIT.
- Des commandes tactiles à l'écran (le bouton en forme d'œil les cache), ou une manette (MFi, PlayStation, Xbox,
  Switch Pro).
- La version européenne se lance dans la langue du téléphone, français ou néerlandais.
- Les vibrations du jeu, dans le téléphone ou dans la manette.
- Les textures plus fines de la version Xbox, si vous fournissez le pack (voir plus bas).
- Les sauvegardes, les réglages et le journal visibles dans l'app Fichiers, sous le nom de l'app.
- Le jeu se met en pause quand on quitte l'app et reprend au retour.

### Ce qu'il faut

- **Votre propre disque du jeu**, sous forme d'image `.iso` ou `.gcm` : rien du jeu n'est fourni ici.
  Deux versions sont acceptées : l'américaine (GGVE78, révision 1) et l'européenne français/néerlandais
  (GGVX78, révision 0). Une image dans un autre format (`.rvz`, `.ciso`...) doit d'abord être convertie en `.iso`,
  par exemple avec Dolphin ou [nod](https://github.com/encounter/nod).
- **Un Mac** avec Xcode et [Rust](https://rustup.rs/).
- Un outil de sideloading pour installer l'app (AltStore, Sideloadly, Feather...), ou votre propre certificat.

Il n'y a pas d'application toute faite à télécharger, et il ne peut pas y en avoir : le programme est fabriqué
à partir du code du jeu qui se trouve sur votre disque. L'app obtenue contient les fichiers de votre disque,
elle est donc pour vos appareils uniquement et ne doit pas être partagée.

### Construction

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

### Bêta : ce qu'il faut savoir

- La version européenne a été essayée sur un iPhone, au doigt. La version américaine n'a pour l'instant tourné
  que sur Mac et au simulateur d'iPhone.
- Les manettes et les vibrations sont écrites mais pas encore essayées sur un vrai appareil.
- Le jeu n'a pas été parcouru jusqu'au bout sur iPhone : des surprises restent possibles.
- La version européenne ouvre toujours sur son choix de langue, comme sur la console.

Un souci ? Ouvrez une *issue* en précisant la version du jeu, votre appareil et ce qui s'est passé. Le fichier
`log.txt`, dans le dossier de l'app (app Fichiers), aide beaucoup.

### Pour les curieux

- `crates/tssm/src/ios.rs` : ce qui est propre à l'iPhone (fichiers, manettes, vibrations, langue).
- `config/addresses.txt` : ce que le port connaît du code du jeu, par nom de symbole. Le traducteur cherche
  chaque nom dans la version du jeu qu'on lui donne, ce qui permet au même programme de se construire pour les
  deux disques.
- `tools/ios.sh` et `tools/ios/` : la construction de l'app.

### D'où ça vient

Petit projet perso : je m'amuse avec une IA (Claude) à faire tourner de vieux jeux sur iPhone et iPad, comme
[l'oncle Ernest](https://github.com/LEBRUNtho/oncle-ernest-ios) ou [Forestia](https://github.com/LEBRUNtho/forestia-ios).
Rien de professionnel, aucune prétention. Tout le mérite du portage d'origine revient à WR3CKINGBALL.

### Droits

Le jeu, ses personnages, images, musiques, voix et textes appartiennent à leurs ayants droit (Nickelodeon et
Viacom, THQ, Heavy Iron Studios, Nintendo). Ce projet n'a aucun lien avec eux, n'est pas officiel, ne fournit
aucun fichier ni aucun code du jeu et ne rapporte rien à personne.

Si vous êtes ayant droit et que quelque chose ici vous gêne, ouvrez une *issue* : le contenu concerné sera retiré
rapidement, sans discussion.

Le dépôt d'origine ne précise pas de licence : son code reste celui de son auteur. Les ajouts de ce fork lui sont
offerts, il peut en faire ce qu'il veut.

## English

**The GameCube game *The SpongeBob SquarePants Movie* (2004), playable on iPhone and iPad, without an emulator.**
Beta version.

This repository is a fork of [tssm-recomp](https://github.com/WR3CKINGBALL/tssm-recomp) by WR3CKINGBALL, who did
all the hard work: translating the game's code into a native program for Windows and Android. This fork adds
three things:

- the **iPhone and iPad** version;
- the **European version** of the game (French and Dutch), besides the American one;
- macOS as a development target.

The original instructions (Windows, Android) are in [`README-original.md`](README-original.md).

### What you get

- The whole game, full screen and widescreen, with no jailbreak and no JIT.
- Touch controls on screen (the eye button hides them), or a controller (MFi, PlayStation, Xbox, Switch Pro).
- The European version starts in the phone's language, French or Dutch.
- The game's rumble, in the phone or in the controller.
- The Xbox version's finer textures, if you supply the pack (see below).
- Saves, settings and the log show in the Files app, under the app's name.
- The game pauses when you leave the app and carries on when you come back.

### What you need

- **Your own disc of the game**, as an `.iso` or `.gcm` image: nothing of the game is provided here.
  Two versions are accepted: the American one (GGVE78, revision 1) and the European French/Dutch one
  (GGVX78, revision 0). An image in another format (`.rvz`, `.ciso`...) has to be converted to `.iso` first,
  for example with Dolphin or [nod](https://github.com/encounter/nod).
- **A Mac** with Xcode and [Rust](https://rustup.rs/).
- A sideloading tool to install the app (AltStore, Sideloadly, Feather...), or your own certificate.

There is no ready-made app to download, and there cannot be one: the program is made from the game's code,
which is on your disc. The app you get holds your disc's files, so it is for your own devices only and must
not be shared.

### Building

1. Install Xcode and Rust, then add the iPhone target: `rustup target add aarch64-apple-ios`
2. In the repository's folder: `tools/ios.sh path/to/the/disc.iso ipa`
3. Install `build/ios/SpongeBobMovie-<version>.ipa` with your sideloading tool.

The first build takes a few minutes. Optionally, in front of the command:

- `PACK=path/to/the/pack.zip` packages a texture pack in Dolphin's format, for example
  [the Xbox version's](https://gamebanana.com/mods/648590). The game's text is then sharper too.
- `ICON=picture.png` replaces the icon, which is otherwise SpongeBob's badge taken from the disc.
- `APP_NAME="..."` changes the name shown under the icon.

The top of [`tools/ios.sh`](tools/ios.sh) explains how to sign with your own certificate and how to try the
app in the simulator.

### Beta: what to know

- The European version has been tried on an iPhone, by touch. The American version has only run on a Mac and
  in the iPhone simulator so far.
- Controllers and rumble are written but have not been tried on a real device yet.
- The game has not been played through to the end on an iPhone: surprises are possible.
- The European version always opens on its language choice, as on the console.

A problem? Open an issue saying which version of the game, which device and what happened. The `log.txt` file,
in the app's folder (Files app), helps a lot.

### For the curious

- `crates/tssm/src/ios.rs`: what is specific to the iPhone (files, controllers, rumble, language).
- `config/addresses.txt`: what the port knows of the game's code, by symbol name. The recompiler looks each
  name up in the version of the game it is given, which lets the same program build for both discs.
- `tools/ios.sh` and `tools/ios/`: the app's build.

### Where this comes from

A small personal project: I have fun with an AI (Claude) getting old games to run on iPhone and iPad, like
[Uncle Ernest](https://github.com/LEBRUNtho/oncle-ernest-ios) or [Forestia](https://github.com/LEBRUNtho/forestia-ios).
Nothing professional, no pretension. All the credit for the original port goes to WR3CKINGBALL.

### Rights

The game, its characters, pictures, music, voices and text belong to their rights holders (Nickelodeon and
Viacom, THQ, Heavy Iron Studios, Nintendo). This project has no link with them, is not official, provides no
file and no code of the game, and earns nobody anything.

If you are a rights holder and something here bothers you, open an issue: the content in question will be
removed quickly, without discussion.

The original repository states no licence: its code remains its author's. This fork's additions are offered to
him, to do with as he likes.
