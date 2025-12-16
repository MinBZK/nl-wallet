{{- define "common.defineProbes" -}}
{{- if not .disableLiveness -}}
livenessProbe:
  httpGet:
    path: /health/live
    port: http
  {{- with .config.readiness }}
  {{- toYaml . | nindent 2 }}
  {{- end }}
{{- end }}
readinessProbe:
  httpGet:
    path: /health/{{ if .useLivenessAsReadiness }}live{{ else }}ready{{ end }}
    port: http
  {{- with .config.liveness }}
  {{- toYaml . | nindent 2 }}
  {{- end }}
startupProbe:
  httpGet:
    path: /health/started
    port: http
  {{- with .config.startup }}
  {{- toYaml . | nindent 2 }}
  {{- end }}
{{- end -}}
