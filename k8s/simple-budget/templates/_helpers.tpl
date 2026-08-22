{{/*
Expand the name of the chart.
*/}}
{{- define "simple-budget.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "simple-budget.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "simple-budget.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Selector labels.

Deliberately just `app`, matching the label the kustomize manifests selected on.
The NetworkPolicy podSelectors key off this, and selectors are immutable once a
Deployment exists, so keep this stable.
*/}}
{{- define "simple-budget.selectorLabels" -}}
app: {{ include "simple-budget.name" . }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "simple-budget.labels" -}}
helm.sh/chart: {{ include "simple-budget.chart" . }}
{{ include "simple-budget.selectorLabels" . }}
app.kubernetes.io/name: {{ include "simple-budget.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Name of one blue/green slot, e.g. simple-budget-blue.
Takes a dict of `root` and `slot`.
*/}}
{{- define "simple-budget.slotName" -}}
{{- printf "%s-%s" (include "simple-budget.fullname" .root) .slot.name | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Selector labels for one blue/green slot.
Takes a dict of `root` and `slot`.
*/}}
{{- define "simple-budget.slotSelectorLabels" -}}
{{ include "simple-budget.selectorLabels" .root }}
version: {{ .slot.name }}
{{- end }}

{{/*
Fully qualified image reference for one slot, digest-pinned when available.
Takes a dict of `root` and `slot`.
*/}}
{{- define "simple-budget.image" -}}
{{- $default := .root.Values.image -}}
{{- $override := default (dict) .slot.image -}}
{{- $repository := $override.repository | default $default.repository -}}
{{- $digest := $override.digest | default $default.digest -}}
{{- if $digest }}
{{- printf "%s@%s" $repository $digest }}
{{- else }}
{{- $tag := $override.tag | default $default.tag | default .root.Chart.AppVersion }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

{{/*
Name of the Secret providing the pod environment.
*/}}
{{- define "simple-budget.secretName" -}}
{{- .Values.secret.existingSecret | default (include "simple-budget.fullname" .) }}
{{- end }}

{{/*
Name of the ConfigMap holding non-secret configuration.
*/}}
{{- define "simple-budget.configMapName" -}}
{{- printf "%s-config" (include "simple-budget.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "simple-budget.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "simple-budget.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
