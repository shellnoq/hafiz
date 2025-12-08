{{/*
Expand the name of the chart.
*/}}
{{- define "hafiz.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "hafiz.fullname" -}}
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
{{- define "hafiz.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "hafiz.labels" -}}
helm.sh/chart: {{ include "hafiz.chart" . }}
{{ include "hafiz.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "hafiz.selectorLabels" -}}
app.kubernetes.io/name: {{ include "hafiz.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "hafiz.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "hafiz.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Get the secret name for credentials
*/}}
{{- define "hafiz.secretName" -}}
{{- if .Values.auth.existingSecret }}
{{- .Values.auth.existingSecret }}
{{- else }}
{{- include "hafiz.fullname" . }}
{{- end }}
{{- end }}

{{/*
Get the database connection string
*/}}
{{- define "hafiz.databaseUrl" -}}
{{- if eq .Values.database.type "sqlite" }}
sqlite://{{ .Values.storage.dataDir }}/{{ .Values.database.sqlitePath }}
{{- else }}
postgres://{{ .Values.database.postgres.username }}:$(POSTGRES_PASSWORD)@{{ .Values.database.postgres.host }}:{{ .Values.database.postgres.port }}/{{ .Values.database.postgres.database }}?sslmode={{ .Values.database.postgres.sslMode }}
{{- end }}
{{- end }}

{{/*
Image name
*/}}
{{- define "hafiz.image" -}}
{{- $tag := default .Chart.AppVersion .Values.image.tag }}
{{- printf "%s:%s" .Values.image.repository $tag }}
{{- end }}

{{/*
PVC name
*/}}
{{- define "hafiz.pvcName" -}}
{{- if .Values.persistence.existingClaim }}
{{- .Values.persistence.existingClaim }}
{{- else }}
{{- include "hafiz.fullname" . }}-data
{{- end }}
{{- end }}

{{/*
Headless service name for cluster mode
*/}}
{{- define "hafiz.headlessServiceName" -}}
{{- include "hafiz.fullname" . }}-headless
{{- end }}

{{/*
Config checksum annotation
*/}}
{{- define "hafiz.configChecksum" -}}
checksum/config: {{ include (print $.Template.BasePath "/configmap.yaml") . | sha256sum }}
checksum/secret: {{ include (print $.Template.BasePath "/secret.yaml") . | sha256sum }}
{{- end }}
