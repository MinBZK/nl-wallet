{{- define "common.urls.combineDomainAndContext" -}}
{{- $domain := index . 0 -}}
{{- $path := index . 1 -}}
{{- if empty $path -}}
{{- printf "https://%s/" $domain -}}
{{- else -}}
{{- printf "https://%s/%s/" $domain $path -}}
{{- end -}}
{{- end -}}
