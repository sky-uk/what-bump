
## Release {{version}} ({{date}})

### ⚠ BREAKING CHANGES
{% for i in breaking -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
### Features
{% for i in features -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
### Bug Fixes
{% for i in fixes -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
### Other Changes
{% for i in other -%}
* {%if i.scope.is_some()%}**{{i.scope.as_ref().unwrap()}}**: {%endif%}{{i.description}}
{% endfor %}
---
