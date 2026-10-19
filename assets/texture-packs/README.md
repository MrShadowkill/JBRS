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

Замініть PNG власними зображеннями, зберігши назви та шляхи. Для найкращого
результату використовуйте квадратні безшовні текстури RGB розміром 64×64,
128×128 або 256×256. `windows.png` у шаблоні вже має темні рами; окремі
світлі вікна й нічне свічення гра відтворює своїм освітленням.

Зараз це **редакторський шаблон**, а не активний завантажувач: рендерер JBRS
ще генерує власний атлас матеріалів і не імпортує сторонні PNG або ZIP у гру.
Тому цей архів можна редагувати та поширювати як заготовку, але заміна файлів
поки не змінить графіку гри.

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

Replace the PNG files with your own images while keeping their names and paths.
For best results, use seamless square RGB textures at 64×64, 128×128, or
256×256. The sample `windows.png` includes dark frames; JBRS handles lit
windows and their night glow through its lighting.

This is currently an **authoring template**, not a runtime loader: JBRS still
generates its material atlas internally and does not import third-party PNG or
ZIP texture packs into the game. You can edit and share this starter archive,
but replacing these files will not change the game's visuals yet.
