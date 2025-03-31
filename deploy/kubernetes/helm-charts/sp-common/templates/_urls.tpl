{{- define "common.urls.combineDomainAndContext" -}}
{{- $domain := index . 0 -}}
{{- $path := index . 1 -}}
{{- if not (hasPrefix "https://" $domain) -}}
{{- $domain = printf "https://%s" $domain -}}
{{- end -}}
{{- if not (hasPrefix "/" $path) -}}
{{- $path = printf "/%s/" $path -}}
{{- end -}}
{{- printf "%s%s" (trimSuffix "/" $domain) $path -}}
{{- end -}}
