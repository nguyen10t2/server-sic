# Node Lifecycle

## Startup

BOOT

↓

LOAD CONFIG

↓

CONNECT WIFI

↓

CONNECT MQTT

↓

SUBSCRIBE TOPICS

↓

SEND HEARTBEAT

↓

READY

---

## Operational

READY

↓

READ SENSOR

↓

PUBLISH SENSOR

↓

WAIT ROUTE

↓

DISPLAY ROUTE

↓

READY

---

## MQTT Lost

READY

↓

MQTT TIMEOUT

↓

USE ROUTE CACHE

↓

RECONNECT MQTT

↓

READY

---

## Network Lost

READY

↓

WIFI LOST

↓

FOLLOW STATIC EXIT

↓

RECONNECT WIFI

↓

READY

---

## Fatal Error

READY

↓

CONFIG ERROR

↓

SAFE MODE