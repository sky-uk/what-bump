
## Release {{version}} ({{date}})

{% if breaking -%}
### ⚠ BREAKING CHANGES
{% for i in breaking -%}
* {%if i.scope%}**{{i.scope}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
{%- if features -%}
### Features
{% for i in features -%}
* {%if i.scope%}**{{i.scope}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
{%- if fixes -%}
### Bug Fixes
{% for i in fixes -%}
* {%if i.scope%}**{{i.scope}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
{%- if other -%}
### Other Changes
{% for i in other -%}
* {%if i.scope%}**{{i.scope}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
---
