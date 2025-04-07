#show link: underline

#set par(justify: true)
#set page(
    "us-letter",
    margin: 1cm,
    footer: context [
        #set align(right)
        #set text(8pt)
        #counter(page).display("1 de 1", both: true)
    ]
)

#set align(center)
#emph(text(font:"TeX Gyre Heros", size: 25pt)[
    #link("https://pet-info.link")[Pet-Info]
])

#set align(start)
#text(font:"PT Sans", size: 12pt, style: "italic", weight: "medium")[
    Nombre: *{{ pet_name | upper }}* \
    Fecha Nacimiento: {{ birthday | date(format="%v") }} ({{ age }}) \
    Raza: {{breed}}
    {% if is_female %}hembra{% else %}macho{% endif %}(
    {% if is_female and is_spaying_neutering %} esterilizada
    {% elif not is_female and is_spaying_neutering %} esterilizado
    {% else %} sin esterelizar
    {% endif %}) \
    Link Público: {{ pet_link}}
]

#set align(center)

= Vacunas
#table(
    columns: (1fr, auto),
    align: (left, center),
    stroke: none,
    table.header(
        [*Descripción*], [*Fecha*]
    ),
    table.hline(),
    {% for vaccine in vaccines %}
    [{{ vaccine.description }}], [{{ vaccine.created_at | date(format="%v") }}],
    table.hline(stroke: 0.5pt),
    {% endfor %}
    table.hline(),
)

#pagebreak()
= Desparasitaciones
#table(
    columns: (1fr, auto),
    align: (left, center),
    stroke: none,
    table.header(
        [*Descripción*], [*Fecha*]
    ),
    table.hline(),
    {% for deworm in deworms %}
    [{{ deworm.description }}], [{{ deworm.created_at | date(format="%v") }}],
    table.hline(stroke: 0.5pt),
    {% endfor %}
    table.hline(),
)

#pagebreak()
= Peso
#table(
    columns: (auto, auto, auto),
    align: (left, center, right),
    stroke: none,
    table.header(
        [*Peso (kg)*], [*Fecha*], [*Edad*]
    ),
    table.hline(),
    {% for weight in weights %}
    [{{ weight.value }}], [{{ weight.created_at | date(format="%v") }}], [{{ weight.fmt_age }}],
    table.hline(stroke: 0.5pt),
    {% endfor %}
    table.hline(),
)

#pagebreak()
= Notas
#set par(
  justify: true,
)
{% for note in notes %}

#set align(center)
== {{ note.title | title }} \
__{{ note.created_at | date(format="%v") }}__

#set align(start)
{{ note.content }}
{% endfor %}
