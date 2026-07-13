{{- define "common.defineProbes" -}}
{{- $port := .port | default "http" -}}
{{- if not .disableLiveness -}}
livenessProbe:
  httpGet:
    path: /health/live
    port: {{ $port }}
  {{- with .config.liveness }}
  {{- toYaml . | nindent 2 }}
  {{- end }}
{{- end }}
readinessProbe:
  httpGet:
    path: /health/{{ if .useLivenessAsReadiness }}live{{ else }}ready{{ end }}
    port: {{ $port }}
  {{- with .config.readiness }}
  {{- toYaml . | nindent 2 }}
  {{- end }}
startupProbe:
  httpGet:
    path: /health/started
    port: {{ $port }}
  {{- with .config.startup }}
  {{- toYaml . | nindent 2 }}
  {{- end }}
{{- end -}}
