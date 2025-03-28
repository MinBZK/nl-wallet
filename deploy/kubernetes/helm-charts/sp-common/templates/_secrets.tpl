{{- define "common.secrets.getSecretName" -}}
{{- $release := index . 0 -}}
{{- $secretName := index . 1 -}}
{{- printf "%s-%s" $release $secretName -}}
{{- end -}}

{{- define "common.secrets.getSecret" -}}
{{ $secret := lookup "v1" "Secret" .release.Namespace (print .release.Name "-" .name) }}
{{ $keyField := .key | default "password" }}
{{- $useSecret := (randAlphaNum 32) }}
    {{- if $secret }}
      {{- $useSecret = index $secret.data $keyField | b64dec }}
    {{- end }}
{{- print $useSecret -}}
{{- end -}}

{{- define "common.secrets.getSecretByName" -}}
{{ $secret := lookup "v1" "Secret" .release.Namespace (print .name) }}
{{ $keyField := .key | default "password" }}
{{- $useSecret := (randAlphaNum 32) }}
    {{- if $secret }}
      {{- $useSecret = index $secret.data $keyField | b64dec }}
    {{- end }}
{{- print $useSecret -}}
{{- end -}}
