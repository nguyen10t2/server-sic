# MQTT Topics Specification

## sensor/{node_id}

Publisher

- Node

Subscriber

- Backend

QoS

- 1

Retain

- false

---

## heartbeat/{node_id}

Publisher

- Node

Subscriber

- Backend

QoS

- 0

Retain

- false

---

## route/{node_id}

Publisher

- Backend

Subscriber

- Node

QoS

- 1

Retain

- true

---

## config/{node_id}

Publisher

- Backend

Subscriber

- Node

QoS

- 1

Retain

- true

---

## alert/{node_id}

Publisher

- Backend

Subscriber

- Node