import { describe, expect, it } from 'vitest';
import { WebSocket } from 'ws';
import { createBroadcaster } from '../src/broadcaster.js';

class FakeSocket {
  public messages: string[] = [];

  readyState: number = WebSocket.OPEN;

  OPEN = WebSocket.OPEN;

  send(payload: string) {
    this.messages.push(payload);
  }
}

class ClosedSocket extends FakeSocket {
  readyState = WebSocket.CLOSED;

  send() {
    throw new Error('should not send when closed');
  }
}

describe('broadcaster', () => {
  it('delivers messages to registered sockets and stops after unregistering', () => {
    const socket = new FakeSocket();
    const second = new FakeSocket();
    const broadcaster = createBroadcaster();
    broadcaster.register('session', socket as unknown as WebSocket);
    broadcaster.register('session', socket as unknown as WebSocket);
    broadcaster.register('session', second as unknown as WebSocket);
    broadcaster.broadcast('session', { hello: 'world' });
    expect(socket.messages).toEqual([JSON.stringify({ hello: 'world' })]);
    expect(second.messages).toEqual([JSON.stringify({ hello: 'world' })]);
    broadcaster.unregister('session', socket as unknown as WebSocket);
    broadcaster.broadcast('session', { ignored: true });
    expect(socket.messages).toHaveLength(1);
    expect(second.messages).toHaveLength(2);
    broadcaster.unregister('session', second as unknown as WebSocket);
    broadcaster.broadcast('session', { after: 'cleanup' });
    expect(second.messages).toHaveLength(2);
    broadcaster.broadcast('unknown', { should: 'ignore' });
    expect(socket.messages).toHaveLength(1);
  });

  it('ignores sockets that are not open', () => {
    const closed = new ClosedSocket();
    const broadcaster = createBroadcaster();
    broadcaster.register('session', closed as unknown as WebSocket);
    broadcaster.broadcast('session', { nope: true });
    expect(closed.messages).toHaveLength(0);
  });

  it('safely handles unregister calls for unknown sessions', () => {
    const broadcaster = createBroadcaster();
    const socket = new FakeSocket();
    expect(() =>
      broadcaster.unregister('missing', socket as unknown as WebSocket),
    ).not.toThrow();
  });

  it('handles socket send errors gracefully', () => {
    class ErrorSocket extends FakeSocket {
      send(payload: string) {
        throw new Error('Network error');
      }
    }
    const errorSocket = new ErrorSocket();
    const normalSocket = new FakeSocket();
    const broadcaster = createBroadcaster();
    broadcaster.register('session', errorSocket as unknown as WebSocket);
    broadcaster.register('session', normalSocket as unknown as WebSocket);

    expect(() => {
      broadcaster.broadcast('session', { test: 'error' });
    }).not.toThrow();

    // The errorSocket should not have received the message
    expect(errorSocket.messages).toHaveLength(0);
    // The normalSocket should have received the message
    expect(normalSocket.messages).toHaveLength(1);
  });
});
