{{- define "ectusr2.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{ default (include "ectusr2.fullname" .) .Values.serviceAccount.name }}
{{- else -}}
{{ default "default" .Values.serviceAccount.name }}
{{- end -}}
{{- end -}}
