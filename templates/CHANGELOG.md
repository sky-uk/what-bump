
## Release {{version}} ({{date}})

{%- if !breaking.is_empty() -%}
### ⚠ BREAKING CHANGES
{% for i in breaking -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
{%- if !features.is_empty() -%}
### Features
{% for i in features -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
{%- if !fixes.is_empty() -%}
### Bug Fixes
{% for i in fixes -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
{%- if !other.is_empty() -%}
### Other Changes
{% for i in other -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
{% endif %}
---
