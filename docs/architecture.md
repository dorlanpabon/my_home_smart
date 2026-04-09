# Architecture Notes

## Decisiones de arquitectura

- `Tauri v2 + Rust + Vite + TypeScript vanilla`.
  - Motivo: mantener la app ligera, con IPC claro y sin meter un framework UI innecesario.
- Backend primero.
  - La lógica crítica está en Rust: auth, firma, cliente HTTP, persistencia y normalización.
- Modelo de dominio estable antes de UI.
  - La UI trabaja sobre `Device` y `DeviceChannel`, no sobre respuestas raw de Tuya.
- Persistencia local simple.
  - Archivos JSON/JSONL en el directorio de datos de la app; suficiente para el MVP y fácil de migrar luego.
- Extensibilidad por capas.
  - `commands` sólo expone transporte Tauri.
  - `services/tuya` contiene la lógica reusable.
  - Los módulos futuros deben colgarse del mismo servicio, no duplicar lógica HTTP.

## Modelo de dominio

### Device

- `id`
- `name`
- `online`
- `category`
- `productId`
- `inferredType`
- `gangCount`
- `channels: DeviceChannel[]`
- `raw`
- `metadata`

### DeviceChannel

- `code`
- `displayName`
- `index`
- `currentState`
- `controllable`
- `alias`

### TuyaFunction

- `code`
- `valueType`
- `values`
- `mode`
- `support`
- `name`
- `description`

### TuyaStatus

- `code`
- `value`

### AppConfig

- `clientId`
- `clientSecret`
- `baseUrl`
- `regionLabel`

### DeviceAlias

- `deviceId`
- `alias`

### ChannelAlias

- `deviceId`
- `channelCode`
- `alias`

## Flujo principal

1. Frontend llama `load_bootstrap`.
2. Rust carga config, metadata y action log.
3. Si hay config, `TuyaService` obtiene token y lista dispositivos.
4. Por cada dispositivo se intenta obtener `details`, `status`, `functions`, `capabilities` y `specification`.
5. `normalize_device(...)` construye el `Device`.
6. `infer_device_channels(...)` detecta los gangs y produce `DeviceChannel[]`.
7. El frontend renderiza tarjetas y controles por canal.

## Cliente Tuya

Capas:

- `signing.rs`
  - genera `Content-SHA256`
  - arma `stringToSign`
  - firma HMAC-SHA256
- `http_client.rs`
  - construye requests firmados
  - agrega headers `client_id`, `sign`, `t`, `nonce`, `access_token`
  - parsea envelopes de Tuya
- `auth.rs`
  - obtiene token
  - cachea token en memoria
  - refresca cuando expira
- `service.rs`
  - orquesta endpoints
  - aplica retry ante token vencido
  - expone métodos de dominio

## Estrategia de detección de gangs

La detección está centralizada en `infer_device_channels(device_id, functions, status, capabilities, metadata)`.

Reglas:

1. Si existen `switch_1..switch_4`, esos códigos mandan.
2. Si no existen, se prueba `switch`.
3. Si tampoco existe, se prueba `switch_led`.
4. Si nada de lo anterior aparece, se buscan códigos booleanos que contengan `switch`.

Detalles importantes:

- La detección revisa `functions`, `status` y `capabilities`.
- `status` define el estado actual.
- `functions/capabilities` definen si el canal es controlable.
- Si hay estado sin capacidad/función, el canal se muestra como `read only`.
- Si existe alias local para un canal, reemplaza el nombre visual.

Ejemplos:

- `switch_1`, `switch_2` => 2 gangs
- `switch_1`, `switch_2`, `switch_3` => 3 gangs
- `switch_1` a `switch_4` => 4 gangs
- sólo `switch` => 1 canal
- sólo `switch_led` => 1 canal

## UI y estado

La UI usa un store simple (`AppStore`) con:

- bootstrap
- config draft
- connection status
- devices
- filtros
- view mode
- busy state por canal
- toasts
- action log

No hay framework de componentes pesado. Se usa renderizado por template string y delegación de eventos.

## Persistencia

Archivos:

- `config.json`
  - credenciales y base URL
- `metadata.json`
  - alias de dispositivos
  - alias de canales
  - preferencias UI
- `actions.jsonl`
  - historial ligero de acciones

## Manejo de errores

Errores cubiertos:

- configuración faltante
- configuración inválida
- fallo HTTP
- error de API Tuya
- token expirado
- respuesta inesperada
- errores de I/O local

Los comandos Tauri devuelven `AppErrorPayload` serializable para que el frontend muestre mensajes claros sin acoplarse a `reqwest` o a errores internos de Rust.

## Roadmap técnico

### Fase actual

- configuración
- test de conexión
- listado
- normalización
- control por canal
- alias locales
- action log

### Siguiente

- refresh automático opcional
- mejor telemetría local
- más heurísticas por categorías Tuya
- pruebas con respuestas reales exportadas

### Posterior

- `scheduler`
- `automation`
- `local_api`
- escenas
- integración con Home Assistant / Node-RED

## Backlog técnico inicial

- soportar paginación robusta en todos los endpoints de listado
- capturar y cachear `details` parciales para mejorar arranque
- persistir último snapshot de dispositivos para modo degradado offline
- exponer más metadata útil de producto en la tarjeta
- añadir pruebas de integración con fixtures de respuestas Tuya reales

## Módulos futuros

### automation

Responsabilidad futura:

- reglas del tipo `if trigger then action`
- escenas simples
- acciones encadenadas

Contrato recomendado:

- no llamar a comandos Tauri
- usar el mismo servicio de aplicación que hoy usa el desktop

### scheduler

Responsabilidad futura:

- disparar acciones por horario
- almacenar tareas
- manejar retry y estados de ejecución

Contrato recomendado:

- programar acciones de dominio, no eventos de UI

### local_api

Responsabilidad futura:

- exponer endpoints HTTP locales para integraciones

Ejemplos:

- `POST /devices/:id/channels/:channelCode/on`
- `POST /devices/:id/channels/:channelCode/off`
- `GET /devices`

## Cómo crecer a HTTP local sin romper el MVP

La clave es mantener el backend actual como un servicio de dominio reutilizable.

Hoy:

- `frontend -> Tauri commands -> TuyaService`

Mañana:

- `frontend -> Tauri commands -> ApplicationService -> TuyaService`
- `local HTTP server -> ApplicationService -> TuyaService`
- `scheduler -> ApplicationService -> TuyaService`

Eso evita duplicar:

- auth
- firma
- normalización
- lógica de canales
- manejo de errores Tuya

La recomendación para la siguiente etapa es extraer una capa `application` o `use_cases` con operaciones como:

- `load_devices()`
- `toggle_channel(device_id, channel_code, value)`
- `rename_device(...)`
- `rename_channel(...)`

Así Tauri deja de ser el centro y pasa a ser sólo un adaptador de transporte.
