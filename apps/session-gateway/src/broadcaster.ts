import type { WebSocket } from 'ws';

type Message = Record<string, unknown>;

const toJson = (payload: Message) => JSON.stringify(payload);

const sendSafe = (socket: WebSocket, payload: Message) => {
  if (socket.readyState === socket.OPEN) {
    socket.send(toJson(payload));
  }
};

export interface Broadcaster {
  register(sessionId: string, socket: WebSocket): void;
  unregister(sessionId: string, socket: WebSocket): void;
  broadcast(sessionId: string, payload: Message): void;
}

export const createBroadcaster = (): Broadcaster => {
  const channels = new Map<string, Set<WebSocket>>();

  const ensureChannel = (sessionId: string) => {
    const existing = channels.get(sessionId);
    if (existing) {
      return existing;
    }
    const created = new Set<WebSocket>();
    channels.set(sessionId, created);
    return created;
  };

  return {
    register(sessionId, socket) {
      ensureChannel(sessionId).add(socket);
    },
    unregister(sessionId, socket) {
      const channel = channels.get(sessionId);
      if (!channel) {
        return;
      }
      channel.delete(socket);
      if (channel.size === 0) {
        channels.delete(sessionId);
      }
    },
    broadcast(sessionId, payload) {
      const channel = channels.get(sessionId);
      if (!channel) {
        return;
      }
      channel.forEach((socket) => sendSafe(socket, payload));
    },
  };
};
