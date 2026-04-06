# Backend WebSocket Integration

## Overview

Implementación de soporte WebSocket en el servidor Rust (Axum) para permitir comunicación bidireccional entre el cliente frontend (Quasar) y el servicio de IA (horbyta_ai) vía gRPC.

## Changes Implemented

### 1. **Cargo.toml** - Dependencias WebSocket

- ✅ Agregada feature `ws` a axum (v0.8.8)
- ✅ Agregada dependencia `futures` para manejo de streams
- ✅ Versión build-dependencies: tonic-build v0.11 (para compilar proto)

### 2. **src/main.rs** - Router WebSocket

- ✅ Rutas independientes: `/api/v1` (REST API) y `/ws` (WebSocket)
- ✅ Ruta WebSocket: `GET /ws` (upgrade protocol HTTP → WebSocket)
- ✅ État compartida entre rutas (AppState)

### 3. **src/error.rs** - Tipos de Error actualizado

- ✅ Nuevo tipo de error: `InternalError` (sin parámetros)
- ✅ Alias de tipo: `ApiError = AppError` para compatibilidad
- ✅ Match correcto en IntoResponse para `InternalError`

### 4. **src/controllers/mod.rs** - Nuevo módulo WebSocket

- ✅ Agregar `pub mod websocket;`

### 5. **src/controllers/websocket.rs** - Controlador WebSocket (NUEVO)

**Autenticación:**

- Extrae JWT desde query params: `?token=...&session_id=...`
- Valida JWT usando la misma llave secreta que REST API
- Extrae user_id de los claims JWT

**Manejo de Conexiones:**

- Handler: `websocket_handler` - acepta upgrade de WebSocket
- Procesa mensajes concurrentemente en tokio::spawn
- Logging de conexión/desconexión con user_id y session_id

**Protocolo de Mensajes:**

```json
{
    "type": "chat|ping|close",
    "data": {
        /* payload */
    },
    "id": "optional-message-id"
}
```

**Tipos de Mensajes:**

- `chat`: Procesa mensaje de chat (TODO: conectar con gRPC)
- `ping`: Echo ping→pong para mantener conexión viva
- `close`: Cierra elegantemente la conexión

**Conversión de tipos:**

- WebSocket Message → JSON
- Usa `Utf8Bytes` (no String) para Message::Text en Axum 0.8.8

## Frontend Updates

### 6. **frontend/src/composables/useWebSocket.ts** - Composable mejorado

**Opciones de inicialización:**

```typescript
interface UseWebSocketOptions {
    path?: string; // default: '/ws'
    token?: string; // JWT token (required para auth)
    sessionId?: string; // ID de sesión (generado si no existe)
    autoConnect?: boolean; // default: true
    autoReconnect?: boolean; // default: true
}
```

**Características:**

- ✅ Autenticación: Incluye token como query param (`?token=...`)
- ✅ Auto-reconexión: Exponential backoff (2^n segundos)
- ✅ Gestión de ciclo de vida: onMounted/onUnmounted
- ✅ Event listener pattern: `on(type, handler)` con cleanup automático
- ✅ Tipado TypeScript strict (sin any)

**Uso en componentes:**

```typescript
const ws = useWebSocket({
    token: authToken,
    sessionId: "user-session-123",
});

ws.on("message", (data) => console.log("Received:", data));
ws.send({ type: "chat", data: { content: "Hola" } });
```

## Arquitectura de integración (Próxima fase)

```
Frontend (Vue/Quasar)
    ↓ WebSocket JSON Messages
Rust Backend (Axum)
    ↓ Conexión gRPC (metadata headers)
Python Service (horbyta_ai)
    ↓ Stream de protobuf ChatEvents
Rust Backend
    ↓ Conversión a JSON
Frontend WebSocket
```

## Próximos pasos

### Fase 2: Integración gRPC

1. Generar stubs gRPC Rust desde proto/horbyta_service.proto
2. Conectar servicio gRPC Python en `handle_chat_message()`
3. Traducir WebSocket payload → ChatRequest protobuf
4. Traducir ChatEvent stream → WebSocket JSON messages
5. Manejar metadata: x-user-id, x-session-id

### Fase 3: Testing end-to-end

1. Test WebSocket básico (ping/pong)
2. Test con token JWT inválido (debe rechazar)
3. Test streaming de respuestas desde gRPC Python
4. Test reconexión automática
5. Test de timeout y errores

## Configuración de puertos

- **Frontend dev server**: 9000 (con proxy `/ws` → `/ws`)
- **Rust Backend**: 127.0.0.1:8890 (configurable via `APP_URI` env var)
- **Python gRPC**: localhost:50051 (en Docker, `horbyta_ai:50051`)

## Validaciones de compilación

✅ `cargo check` pasa sin errores
✅ Tipos WebSocket correctos para Axum 0.8.8
✅ Error handling completo
✅ Logging con tracing crate

## Archivos modificados

- Cargo.toml (agregadas features)
- src/main.rs (rutas WebSocket)
- src/error.rs (tipos de error)
- src/controllers/mod.rs (módulo websocket)
- src/controllers/websocket.rs (NUEVO)
- frontend/src/composables/useWebSocket.ts (autenticación)
