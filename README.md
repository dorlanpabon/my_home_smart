# Tuya Desk Controller

Aplicación de escritorio para Windows construida con `Tauri + Rust + Vite + TypeScript` para controlar switches Tuya/Tuya Smart desde PC, con soporte explícito para dispositivos multigang.

## Qué resuelve

- Configuración local de `TUYA_CLIENT_ID`, `TUYA_CLIENT_SECRET`, `TUYA_BASE_URL` y región.
- Prueba de conexión y obtención de token Tuya Cloud desde Rust.
- Listado de dispositivos enlazados al proyecto.
- Normalización de dispositivos hacia un modelo `Device` y `DeviceChannel`.
- Detección centralizada de canales `switch`, `switch_led`, `switch_1` a `switch_4`.
- Control ON/OFF independiente por canal.
- Refresh manual, feedback de errores y registro local simple de acciones.
- Alias locales de dispositivos y canales.
- Base preparada para `automation`, `scheduler` y `local_api`.

## Stack

- Tauri v2
- Rust backend
- Vite + TypeScript vanilla frontend
- IPC con comandos Tauri
- Persistencia local en archivos JSON/JSONL

## Requisitos

- Node.js 22+
- npm 10+
- Rust `stable`
- Windows con toolchain MSVC

Validado en esta máquina con `rustc 1.94.1`.

## Comandos

```bash
npm install
npm run tauri dev
npm run tauri build
```

Comandos útiles adicionales:

```bash
npm run test
npm run build
cd src-tauri && cargo test
```

## Flujo de uso

1. Abrir la app.
2. Si no existe configuración, se muestra la pantalla de setup.
3. Guardar credenciales Tuya Cloud y región.
4. Probar conexión.
5. Cargar dispositivos.
6. Revisar canales detectados por tarjeta.
7. Encender o apagar cada canal de forma independiente.
8. Refrescar estados cuando haga falta.

## Configuración Tuya

Valores típicos para tu caso:

- `Base URL`: `https://openapi.tuyaus.com`
- `Region label`: `Western America Data Center`

La app guarda la configuración en el directorio de datos de la aplicación. Si abres Settings más tarde y dejas vacío el `Client Secret`, se conserva el secreto ya guardado.

## Estructura

```text
src/
  components/
  pages/
  services/
  stores/
  styles/
  types/
  utils/

src-tauri/
  src/
    commands/
    config/
    errors/
    future/
    models/
    services/
  capabilities/
  icons/
```

## Arquitectura

Resumen corto:

- El frontend nunca consume el JSON raw de Tuya para renderizar lógica principal.
- Rust encapsula firma, token, HTTP y normalización.
- El comando Tauri devuelve `Device[]` ya listos para UI.
- La detección de gangs está en una función reusable: `infer_device_channels(...)`.
- Los módulos futuros deberán reutilizar el mismo servicio de dominio y no hablar con la UI.

Detalle completo en [docs/architecture.md](/D:/xampp/htdocs/my_home_smart/docs/architecture.md).

## Persistencia local

Se guarda en el directorio de datos de la app:

- `config.json`: credenciales y URL base
- `metadata.json`: alias y preferencias UI
- `actions.jsonl`: historial simple de acciones

## Estrategia de detección de gangs

Orden de prioridad:

1. `switch_1`, `switch_2`, `switch_3`, `switch_4`
2. `switch`
3. `switch_led`
4. fallback por códigos booleanos que contengan `switch`

Si un código existe sólo en `status`, el canal se muestra pero queda como no controlable.

## Build generado

La build validada genera el instalador NSIS en:

[`src-tauri/target/release/bundle/nsis/Tuya Desk Controller_0.1.0_x64-setup.exe`](/D:/xampp/htdocs/my_home_smart/src-tauri/target/release/bundle/nsis/Tuya%20Desk%20Controller_0.1.0_x64-setup.exe)

## Próximos pasos naturales

- Probar con tus dispositivos reales y ajustar mensajes según respuestas concretas de Tuya.
- Afinar el fallback de endpoints de listado si tu proyecto usa un paquete/permisos específicos.
- Añadir polling opcional o refresh programado.
- Incorporar `local_api` y `scheduler` sobre el mismo `TuyaService`.

## Notas

- Se usa la API oficial de Tuya Cloud.
- No hay scraping ni dependencia de la app móvil.
- El MVP está orientado a switches de luz, especialmente multigang.
