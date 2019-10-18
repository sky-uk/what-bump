# Changelog

## {{version}} ({{date}})

### ⚠ BREAKING CHANGES
{% for i in breaking -%}
* {{i.description}}
{% endfor %}
### Features
{% for i in features -%}
* {{i.description}}
{% endfor %}
### Bug Fixes
{% for i in fixes -%}
* {{i.description}}
{% endfor %}
