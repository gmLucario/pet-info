// Pet Info Report - Modern Card Design
#set page(
    width: 210mm,  // A4 width
    height: auto,
    margin: (x: 2cm, y: 2cm),
    fill: rgb("#f8fafc"), // Light background
    footer: context [
        #set align(center)
        #set text(9pt, fill: rgb("#64748b"))
        #link("https://pet-info.link")[Pet-Info] • Página #counter(page).display("1 de 1", both: true)
    ]
)

#set text(
    font: "PT Sans",
    size: 11pt,
    fill: rgb("#1e293b")
)

#set par(justify: true)
#show link: underline

// ==================== HEADER SECTION ====================
#set align(center)

// Pet Picture (if available)
{% if has_picture %}
#box(
    width: 100%,
    height: 150pt,
    radius: 12pt,
    clip: true,
)[
    #image("{{ image_filename }}", width: 100%, height: 100%, fit: "cover")
]
#v(15pt)
{% endif %}

// Pet Name
#text(
    size: 32pt,
    weight: "bold",
    fill: rgb("#0f172a")
)[
    {{ pet_name | upper }}
]

#v(10pt)

// Basic Info Pill
#box(
    fill: rgb("#e0e7ff"),
    inset: (x: 20pt, y: 10pt),
    radius: 20pt
)[
    #text(
        size: 12pt,
        fill: rgb("#3730a3"),
        weight: "medium"
    )[
        {{ breed }} •
        {% if is_female %}Hembra{% else %}Macho{% endif %} •
        {{ age }}
    ]
]

#v(20pt)

// ==================== BASIC INFO CARD ====================
#set align(left)

#block(
    fill: white,
    inset: 20pt,
    radius: 12pt,
    width: 100%,
    stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)
)[
    #text(size: 16pt, weight: "bold", fill: rgb("#0f172a"))[Información General]
    #v(12pt)

    #grid(
        columns: (auto, 1fr),
        row-gutter: 8pt,
        column-gutter: 12pt,

        [*Fecha de Nacimiento:*], [{{ birthday | date(format="%v") }}],
        [*Edad:*], [{{ age }}],
        [*Raza:*], [{{ breed }}],
        [*Sexo:*], [{% if is_female %}Hembra{% else %}Macho{% endif %}],
        [*Esterilización:*], [
            {% if is_female and is_spaying_neutering %}[+] Esterilizada
            {% elif not is_female and is_spaying_neutering %}[+] Esterilizado
            {% else %}[-] Sin esterilizar
            {% endif %}
        ],
        [*Link Público:*], [#link("{{ pet_link }}")[Ver perfil]]
    )
]

#v(20pt)

// ==================== VACCINES SECTION ====================
#block(
    fill: white,
    inset: 20pt,
    radius: 12pt,
    width: 100%,
    stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)
)[
    #text(size: 16pt, weight: "bold", fill: rgb("#0f172a"))[Vacunas]
    #v(12pt)

    #table(
        columns: (1fr, auto),
        align: (left, center),
        stroke: none,
        row-gutter: 8pt,
        table.header(
            [*Descripción*],
            [*Fecha*]
        ),
        table.hline(stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)),
        {% for vaccine in vaccines %}
        [{{ vaccine.description }}],
        [#text(fill: rgb("#64748b"))[{{ vaccine.created_at | date(format="%v") }}]],
        table.hline(stroke: (paint: rgb("#f1f5f9"), thickness: 0.5pt)),
        {% endfor %}
    )
]

#v(20pt)

// ==================== DEWORMS SECTION ====================
#block(
    fill: white,
    inset: 20pt,
    radius: 12pt,
    width: 100%,
    stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)
)[
    #text(size: 16pt, weight: "bold", fill: rgb("#0f172a"))[Desparasitaciones]
    #v(12pt)

    #table(
        columns: (1fr, auto),
        align: (left, center),
        stroke: none,
        row-gutter: 8pt,
        table.header(
            [*Descripción*],
            [*Fecha*]
        ),
        table.hline(stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)),
        {% for deworm in deworms %}
        [{{ deworm.description }}],
        [#text(fill: rgb("#64748b"))[{{ deworm.created_at | date(format="%v") }}]],
        table.hline(stroke: (paint: rgb("#f1f5f9"), thickness: 0.5pt)),
        {% endfor %}
    )
]

#pagebreak()

// ==================== WEIGHT HISTORY ====================
#block(
    fill: white,
    inset: 20pt,
    radius: 12pt,
    width: 100%,
    stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)
)[
    #text(size: 16pt, weight: "bold", fill: rgb("#0f172a"))[Historial de Peso]
    #v(12pt)

    #table(
        columns: (auto, auto, 1fr),
        align: (left, center, right),
        stroke: none,
        row-gutter: 8pt,
        table.header(
            [*Peso (kg)*],
            [*Fecha*],
            [*Edad*]
        ),
        table.hline(stroke: (paint: rgb("#e2e8f0"), thickness: 1pt)),
        {% for weight in weights %}
        [*{{ weight.value }}*],
        [#text(fill: rgb("#64748b"))[{{ weight.created_at | date(format="%v") }}]],
        [#text(fill: rgb("#64748b"), style: "italic")[{{ weight.fmt_age }}]],
        table.hline(stroke: (paint: rgb("#f1f5f9"), thickness: 0.5pt)),
        {% endfor %}
    )
]

#v(20pt)

// ==================== NOTES SECTION ====================
{% for note in notes %}
#block(
    fill: rgb("#fef3c7"),
    inset: 20pt,
    radius: 12pt,
    width: 100%,
    stroke: (paint: rgb("#fbbf24"), thickness: 1pt)
)[
    #set align(left)
    #text(size: 15pt, weight: "bold", fill: rgb("#78350f"))[
        {{ note.title | title }}
    ]
    #v(5pt)
    #text(size: 9pt, fill: rgb("#92400e"), style: "italic")[
        {{ note.created_at | date(format="%v") }}
    ]
    #v(10pt)
    #text(size: 11pt, fill: rgb("#1e293b"))[
        {{ note.content }}
    ]
]
#v(15pt)
{% endfor %}
