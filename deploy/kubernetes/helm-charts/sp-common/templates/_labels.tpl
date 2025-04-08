
{{/*
Kubernetes standard labels
*/}}
{{- define "common.labels.standard.name" -}}
app.kubernetes.io/name: {{ include "common.names.fullname" . }}
{{- end -}}

{{- define "common.labels.standard.release.name" -}}
app.kubernetes.io/instance: "{{ .Release.Name }}"
{{- end -}}

{{- define "common.labels.standard.release.service" -}}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Common labels to be used in templates
*/}}

{{- define "common.labels.standard" -}}
helm.sh/chart: {{ include "common.names.chart" . }}
{{ include "common.labels.standard.name" . }}
{{ include "common.labels.standard.release.name" . }}
{{ include "common.labels.standard.release.service" . }}
{{- end -}}

{{- define "common.labels.component" -}}
{{- $args := . -}}
{{- $component := index $args 0 -}}
{{- $context := index $args 1 -}}
{{ include "common.labels.standard" $context }}
app.kubernetes.io/component: {{ $component }}
{{- end -}}


{{/*
Selector labels
*/}}
{{- define "common.labels.selectorLabels" -}}
{{ include "common.labels.standard.name" . }}
{{ include "common.labels.standard.release.name" . }}
{{- end }}

{{/*
Custom selector labels
*/}}
{{- define "common.labels.componentSelectorLabels" -}}
{{- $args := . -}}
{{- $component := index $args 0 -}}
{{- $context := index $args 1 -}}
{{ include "common.labels.selectorLabels" $context }}
app.kubernetes.io/component: {{ $component }}
{{- end }}
