{{/*
Expand the name of the chart.
*/}}
{{- define "hafiz.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
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
Return the PostgreSQL hostname
*/}}
{{- define "hafiz.postgresql.host" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "hafiz.fullname" .) }}
{{- else }}
{{- .Values.postgresql.external.host }}
{{- end }}
{{- end }}

{{/*
Return the PostgreSQL port
*/}}
{{- define "hafiz.postgresql.port" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "5432" }}
{{- else }}
{{- .Values.postgresql.external.port | toString }}
{{- end }}
{{- end }}

{{/*
Return the PostgreSQL database name
*/}}
{{- define "hafiz.postgresql.database" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.database }}
{{- else }}
{{- .Values.postgresql.external.database }}
{{- end }}
{{- end }}

{{/*
Return the PostgreSQL username
*/}}
{{- define "hafiz.postgresql.username" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.username }}
{{- else }}
{{- .Values.postgresql.external.username }}
{{- end }}
{{- end }}

{{/*
Return the PostgreSQL secret name
*/}}
{{- define "hafiz.postgresql.secretName" -}}
{{- if .Values.postgresql.enabled }}
{{- if .Values.postgresql.auth.existingSecret }}
{{- .Values.postgresql.auth.existingSecret }}
{{- else }}
{{- printf "%s-postgresql" (include "hafiz.fullname" .) }}
{{- end }}
{{- else }}
{{- if .Values.postgresql.external.existingSecret }}
{{- .Values.postgresql.external.existingSecret }}
{{- else }}
{{- printf "%s-postgresql-external" (include "hafiz.fullname" .) }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Return the PostgreSQL secret key
*/}}
{{- define "hafiz.postgresql.secretKey" -}}
{{- if .Values.postgresql.enabled }}
{{- if .Values.postgresql.auth.existingSecret }}
{{- "password" }}
{{- else }}
{{- "password" }}
{{- end }}
{{- else }}
{{- .Values.postgresql.external.existingSecretKey | default "postgresql-password" }}
{{- end }}
{{- end }}

{{/*
Return the auth secret name
*/}}
{{- define "hafiz.auth.secretName" -}}
{{- if .Values.hafiz.auth.existingSecret }}
{{- .Values.hafiz.auth.existingSecret }}
{{- else }}
{{- printf "%s-auth" (include "hafiz.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Return the encryption secret name
*/}}
{{- define "hafiz.encryption.secretName" -}}
{{- if .Values.hafiz.encryption.existingSecret }}
{{- .Values.hafiz.encryption.existingSecret }}
{{- else }}
{{- printf "%s-encryption" (include "hafiz.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Return the admin secret name
*/}}
{{- define "hafiz.admin.secretName" -}}
{{- if .Values.hafiz.admin.existingSecret }}
{{- .Values.hafiz.admin.existingSecret }}
{{- else }}
{{- printf "%s-admin" (include "hafiz.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Return the LDAP secret name
*/}}
{{- define "hafiz.ldap.secretName" -}}
{{- if .Values.hafiz.ldap.existingSecret }}
{{- .Values.hafiz.ldap.existingSecret }}
{{- else }}
{{- printf "%s-ldap" (include "hafiz.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Return the TLS secret name
*/}}
{{- define "hafiz.tls.secretName" -}}
{{- if .Values.hafiz.tls.existingSecret }}
{{- .Values.hafiz.tls.existingSecret }}
{{- else }}
{{- printf "%s-tls" (include "hafiz.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Generate cluster peers list
*/}}
{{- define "hafiz.clusterPeers" -}}
{{- $fullname := include "hafiz.fullname" . -}}
{{- $replicas := int .Values.replicaCount -}}
{{- $gossipPort := int .Values.hafiz.cluster.gossipPort -}}
{{- $peers := list -}}
{{- range $i := until $replicas -}}
{{- $peers = append $peers (printf "%s-%d.%s-headless:%d" $fullname $i $fullname $gossipPort) -}}
{{- end -}}
{{- join "," $peers -}}
{{- end }}
