import { createElement, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './styles.css';

interface DesktopStatus {
  connected: boolean;
  clientId: string | null;
  reason: string | null;
  controlState: string;
}

type ControlOperation = 'desktop_pause' | 'desktop_stop' | 'desktop_takeover';

const initialStatus: DesktopStatus = {
  connected: false,
  clientId: null,
  reason: 'Authenticating native host…',
  controlState: 'authentication-disconnected',
};

async function readStatus(): Promise<DesktopStatus> {
  return invoke<DesktopStatus>('desktop_status');
}

function App() {
  const [status, setStatus] = useState<DesktopStatus>(initialStatus);
  const [controlMessage, setControlMessage] = useState<string>('No autonomous control is active.');

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;

    const refresh = async () => {
      try {
        const next = await readStatus();
        if (!cancelled) {
          setStatus(next);
        }
      } catch (error) {
        if (!cancelled) {
          setStatus({
            connected: false,
            clientId: null,
            reason: String(error),
            controlState: 'authentication-disconnected',
          });
        }
      }
      if (!cancelled) {
        timer = window.setTimeout(refresh, 2000);
      }
    };

    void refresh();
    return () => {
      cancelled = true;
      if (timer !== undefined) {
        window.clearTimeout(timer);
      }
    };
  }, []);

  const invokeControl = async (operation: ControlOperation, label: string) => {
    setControlMessage(`${label} requested…`);
    try {
      await invoke(operation);
      setControlMessage(`${label} accepted by protected control authority.`);
      setStatus(await readStatus());
    } catch (error) {
      setControlMessage(`${label} denied: ${String(error)}`);
    }
  };

  const statusLabel = status.connected ? 'Authenticated local host' : 'Authentication disconnected';
  const statusDetail = status.connected
    ? `Protected desktop client ${status.clientId ?? 'unknown'}`
    : status.reason ?? 'No protected desktop credential is available.';

  return createElement(
    'main',
    { className: 'shell' },
    createElement(
      'section',
      { className: 'hero' },
      createElement('div', { className: 'eyebrow' }, 'GOLAM · LOCAL AGENT OS'),
      createElement('h1', null, 'Computer control stays visible and interruptible.'),
      createElement(
        'p',
        { className: 'lede' },
        'The renderer is an untrusted presentation tier. Credentials, capability material, control leases, and platform handles remain in protected Rust and golamd state.'
      )
    ),
    createElement(
      'section',
      { className: `status-card ${status.connected ? 'connected' : 'disconnected'}` },
      createElement('div', { className: 'status-dot', 'aria-hidden': 'true' }),
      createElement(
        'div',
        null,
        createElement('strong', null, statusLabel),
        createElement('p', null, statusDetail)
      )
    ),
    createElement(
      'section',
      { className: 'control-card' },
      createElement(
        'div',
        { className: 'control-heading' },
        createElement('div', null, createElement('span', { className: 'label' }, 'VISIBLE CONTROL CHANNEL'), createElement('h2', null, status.controlState)),
        createElement('span', { className: 'local-badge' }, 'LOCAL ONLY')
      ),
      createElement(
        'div',
        { className: 'controls' },
        createElement(
          'button',
          {
            className: 'control-button pause',
            disabled: !status.connected,
            onClick: () => void invokeControl('desktop_pause', 'Pause'),
          },
          'Pause'
        ),
        createElement(
          'button',
          {
            className: 'control-button stop',
            disabled: !status.connected,
            onClick: () => void invokeControl('desktop_stop', 'Stop'),
          },
          'Stop'
        ),
        createElement(
          'button',
          {
            className: 'control-button takeover',
            disabled: !status.connected,
            onClick: () => void invokeControl('desktop_takeover', 'Take over'),
          },
          'Take over'
        )
      ),
      createElement('p', { className: 'control-message', role: 'status' }, controlMessage)
    ),
    createElement(
      'section',
      { className: 'principles' },
      createElement('article', null, createElement('span', null, '01'), createElement('h3', null, 'Fail closed'), createElement('p', null, 'Missing or stale authority never becomes permission.')),
      createElement('article', null, createElement('span', null, '02'), createElement('h3', null, 'Human first'), createElement('p', null, 'Pause, stop, and takeover supersede autonomous input.')),
      createElement('article', null, createElement('span', null, '03'), createElement('h3', null, 'Strict local'), createElement('p', null, 'No hidden cloud, remote navigation, or telemetry fallback.'))
    )
  );
}

const root = document.getElementById('root');
if (root === null) {
  throw new Error('Golam desktop root element is missing');
}
createRoot(root).render(createElement(App, {}));
