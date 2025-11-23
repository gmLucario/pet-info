// Pet Public Info Card - Optimized for image/social sharing
#set page(
    width: 800pt,
    height: auto,
    margin: 2cm,
    fill: rgb("#f8fafc"), // Light background similar to web
)

#set text(
    font: "PT Sans",
    size: 11pt,
)

#set align(center)

// Pet Picture (if available)
{% if has_picture %}
#box(
    width: 100%,
    height: 300pt,
    clip: true,
)[
    // Note: In practice, you'd need to load the image from a file or base64
    #text(size: 40pt, fill: gray)[🐾]
    #v(10pt)
    #text(size: 10pt, fill: gray.darken(30%))[Pet Picture]
]
#v(20pt)
{% endif %}

// Pet Name
#text(
    size: 28pt,
    weight: "bold",
    fill: rgb("#0f172a") // Primary color from HTML
)[
    {{ pet_name | title }}
]

#v(10pt)

// Lost Status Alert (if applicable)
{% if is_lost %}
#block(
    fill: rgb("#fecaca"), // Light red background
    inset: 12pt,
    radius: 8pt,
    width: 100%
)[
    #set align(center)
    #text(
        size: 14pt,
        weight: "bold",
        fill: rgb("#991b1b")
    )[
        ⚠️ MASCOTA PERDIDA!
    ]
]
#v(10pt)
{% endif %}

// Basic Info
#text(
    style: "italic",
    size: 12pt,
    fill: rgb("#475569")
)[
    {{ breed }} • {{ sex }}
    {% if weight %} • [{{ weight }} kg]{% endif %}
    • {{ age }}
]

#v(20pt)

// About Section
#set align(left)
#block(
    fill: white,
    inset: 15pt,
    radius: 8pt,
    width: 100%
)[
    #set par(justify: true)
    #text(size: 13pt, weight: "semibold")[Acerca de ...]
    #v(8pt)
    {{ about_pet }}
]

#v(15pt)

// Contact Section (only if lost)
{% if is_lost and contact_info %}
#block(
    fill: white,
    inset: 15pt,
    radius: 8pt,
    width: 100%
)[
    #text(size: 13pt, weight: "semibold")[Contacto]
    #v(8pt)
    {{ contact_info }}
]
#v(15pt)
{% endif %}

// Health Information
#block(
    fill: white,
    inset: 15pt,
    radius: 8pt,
    width: 100%
)[
    #text(size: 13pt, weight: "semibold")[Salud]
    #v(8pt)

    // Spay/Neuter Status
    #grid(
        columns: (auto, 1fr),
        gutter: 10pt,
        [{% if is_spaying_neutering %}☑{% else %}☐{% endif %}],
        [
            #text[Esterilizada]
            #v(5pt)
            #text(
                size: 9pt,
                style: "italic",
                fill: gray.darken(30%)
            )[
                {{ pet_name | lower }}
                {% if is_spaying_neutering %}
                ha sido esterilizada
                {% else %}
                *NO* está esterilizada
                {% endif %}
            ]
        ]
    )

    #v(10pt)

    // Health records info
    #text(size: 10pt, fill: gray.darken(20%))[
        Ver registros completos:
        • Desparasitaciones
        • Vacunas
    ]
]

#v(20pt)

// Footer with link
#set align(center)
#text(
    size: 9pt,
    fill: gray
)[
    Más información en #link("https://pet-info.link")[pet-info.link]
]
