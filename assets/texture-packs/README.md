# Шаблони текстурпаків / Texture pack templates

## Українська

У `C13HPT-Template.zip` є приклад структури текстурпака:

```text
C13HPT-Template/
├── README.md
└── assets/textures/
    ├── walls.png
    ├── windows.png
    ├── floors.png
    ├── roofs.png
    ├── roads.png
    └── metal.png
```

Скопіюйте папку паку або ZIP у директорію текстурпаків і виберіть його у
`SET → GRAPHICS → TEXTURE PACK`. Підтримуються папки та ZIP з PNG у
`assets/textures/` або `textures/`; читання ZIP відбувається без розпакування.
Зображення декодуються при запуску й перебудові атласу, а не щокадру.

Розташування паків:

- Windows: `C:\JBRS\texture-packs\` або поруч із грою в `texture-packs/`.
- Linux: `~/JBRS/texture-packs/` або поруч із грою в `texture-packs/`.
- Android: `/storage/emulated/0/JBRS/texture-packs/`.
- Для розробки можна задати `JBRS_TEXTURE_DIR` як каталог із папками/ZIP.

Перезапустіть гру після додавання паку: список паків читається під час запуску.
У налаштуваннях виберіть його клавішами вліво/вправо. Невалідні або завеликі
PNG показують помилку в журналі, а гра повертається до вбудованих текстур.
Для найкращого результату використовуйте квадратні RGB/RGBA PNG до 2048×2048;
атлас зменшує їх до поточного рівня текстур. На LOW і POTATO текстура стіни,
вікна, дверей і даху також використовується на спрощених фасадах будинків.

## English

`C13HPT-Template.zip` contains an example texture pack layout:

```text
C13HPT-Template/
├── README.md
└── assets/textures/
    ├── walls.png
    ├── windows.png
    ├── floors.png
    ├── roofs.png
    ├── roads.png
    └── metal.png
```

Copy the pack folder or ZIP into a texture-pack directory and select it under
`SET → GRAPHICS → TEXTURE PACK`. Folders and ZIPs containing PNGs under
`assets/textures/` or `textures/` are supported. ZIPs are read without extracting
them. Images are decoded when the game starts or rebuilds the atlas, never per
frame.

Pack directories:

- Windows: `C:\JBRS\texture-packs\` or `texture-packs/` beside the game.
- Linux: `~/JBRS/texture-packs/` or `texture-packs/` beside the game.
- Android: `/storage/emulated/0/JBRS/texture-packs/`.
- Developers can set `JBRS_TEXTURE_DIR` to a folder containing pack folders/ZIPs.

Restart after adding a pack; the pack list is read at startup. Select it with
left/right in graphics settings. Invalid or oversized PNGs are logged and the
game falls back to its built-in atlas. Square RGB/RGBA PNGs up to 2048×2048 are
supported. The atlas scales them to the current texture quality. LOW and POTATO
building facades use the wall, window, door, and roof materials as well.
