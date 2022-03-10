# Changelog

{{ range .Versions }}
## {{ if .Tag.Previous }}[[{{ .Tag.Name }}]({{ $.Info.RepositoryURL }}/compare/{{ .Tag.Previous.Name }}...{{ .Tag.Name }})]{{ else }}[{{ .Tag.Name }}]{{ end }} - {{ datetime "2006-01-02" .Tag.Date }}

{{ range .CommitGroups -}}
### {{ .Title }}
{{ range .Commits -}}{{ if regexMatch `^.+\(\[#[1-9][0-9]*\]\(https://github.com/line/cosmwasm/issues/[1-9][0-9]*\)\)$` .Subject }}
* {{ if .Scope }}**{{ .Scope }}:** {{ end }}{{ mustRegexReplaceAll `issues/([1-9][0-9]*)\)\)$` .Subject `pull/${1}))` }}{{ end }}{{ end }}

{{ end -}}

{{- if .NoteGroups -}}
{{ range .NoteGroups -}}
### {{ .Title }}

{{ range .Notes }}
{{ .Body }}
{{ end }}
{{ end -}}
{{ end -}}
{{ end -}}
