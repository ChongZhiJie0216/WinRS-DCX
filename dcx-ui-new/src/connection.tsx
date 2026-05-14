import React, {useState, useEffect, useCallback} from 'react';
import Button from 'react-bootstrap/Button';
import Form from 'react-bootstrap/Form';
import Spinner from 'react-bootstrap/Spinner';
import Badge from 'react-bootstrap/Badge';
import {toast} from 'react-toastify';
import {FaSync} from 'react-icons/fa';

type ConnectionProps = Record<string, never>;

function Connection(_props: ConnectionProps) {
  const [networks, setNetworks] = useState<string[]>([]);
  const [selected, setSelected] = useState('');
  const [baudRate, setBaudRate] = useState('38400');

  const [ip, setIp] = useState<string | undefined>(undefined);
  const [current, setCurrent] = useState<string | undefined>(undefined);
  const [isLoading, setIsLoading] = useState(false);

  const showFetchError = () => {
    if (!toast.isActive('fetch-failed')) {
      toast.error(`Fetching COM Port status failed.`, {
        position: 'bottom-left',
        toastId: 'fetch-failed',
      });
    }
  };

  const fetchConnection = useCallback(async () => {
    try {
      const response = await fetch('/api/connection', {
        credentials: 'same-origin',
      });
      if (!response.ok) {
        throw new Error(response.statusText);
      }

      const connection = (await response.json()) as {
        current?: string;
        ip?: string;
      };
      setCurrent(connection.current ?? undefined);
      setIp(connection.ip ?? undefined);
      if (connection.current !== undefined) {
        setSelected(connection.current);
      }
    } catch {
      showFetchError();
    }
  }, []);

  const fetchNetworks = useCallback(async () => {
    setIsLoading(true);
    try {
      const response = await fetch('/api/networks', {
        credentials: 'same-origin',
      });
      if (!response.ok) {
        throw new Error(response.statusText);
      }

      const nets = (await response.json()) as string[];
      setNetworks(nets);
      
      setSelected((prevSelected) => {
        if (nets.length > 0 && !prevSelected) {
          return nets[0];
        }
        return prevSelected;
      });
    } catch {
      showFetchError();
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetchConnection();
    void fetchNetworks();
  }, [fetchConnection, fetchNetworks]);

  const updateConnection = async () => {
    setIsLoading(true);
    const formData = new FormData();
    formData.append('ssid', selected);
    formData.append('password', baudRate);
    const data = new URLSearchParams(
      formData as unknown as Record<string, string>,
    );

    try {
      const response = await fetch('/api/connection', {
        method: 'PATCH',
        body: data,
        credentials: 'same-origin',
      });
      if (!response.ok) {
        throw new Error(response.statusText);
      }

      const connection = (await response.json()) as {
        current?: string;
        ip?: string;
      };
      setCurrent(connection.current ?? undefined);
      setIp(connection.ip ?? undefined);
      if (connection.current) {
        setSelected(connection.current);
      }
    } catch {
      setIp('Disconnected');
      toast.error(`Could not connect to port ${selected}`, {
        position: 'bottom-left',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const disconnectConnection = async () => {
    setIsLoading(true);
    try {
      await fetch('/api/connection', {
        credentials: 'same-origin',
        method: 'DELETE',
      });
      void fetchConnection();
    } catch {
      toast.error('Device disconnected', {position: 'bottom-left'});
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    void updateConnection();
  };

  const handleDisconnection = (event: React.FormEvent) => {
    event.preventDefault();
    void disconnectConnection();
  };

  const renderLoadingSpinner = () => (
    <div className="text-center">
      <Spinner animation="border" role="status" variant="primary">
        <span className="visually-hidden">Loading...</span>
      </Spinner>
    </div>
  );

  return (
    <div>
      {!current ? (
        <Form onSubmit={handleSubmit}>
          <Form.Group className="mb-3">
            <Form.Label className="d-flex justify-content-between align-items-center">
              Device / COM Port
              <Button variant="link" size="sm" onClick={() => void fetchNetworks()} disabled={isLoading}>
                <FaSync />
              </Button>
            </Form.Label>
            {isLoading ? renderLoadingSpinner() : (
                <Form.Control
                as="select"
                value={selected}
                onChange={(event) => {
                  setSelected(event.target.value);
                }}
              >
                {networks.length === 0 ? <option>No ports found</option> : networks.sort().map((port) => (
                  <option key={port}>{port}</option>
                ))}
              </Form.Control>
            )}
          </Form.Group>
          <Form.Group className="mb-3">
            <Form.Label>Baud Rate</Form.Label>
            <Form.Control
              as="select"
              value={baudRate}
              onChange={(event) => {
                setBaudRate(event.target.value);
              }}
            >
              {['9600', '19200', '38400', '57600', '115200'].map((rate) => (
                <option key={rate} value={rate}>{rate}</option>
              ))}
            </Form.Control>
          </Form.Group>
          <Button variant="danger" className="w-100" type="submit" disabled={isLoading || networks.length === 0}>
            Connect
          </Button>
        </Form>
      ) : (
        <Form onSubmit={handleDisconnection}>
          <Form.Group className="mb-3">
            <Form.Label>Device / COM Port</Form.Label>
            <Form.Control disabled type="text" value={current} />
          </Form.Group>
          <Form.Group className="mb-3">
            <Form.Label>Connection Status</Form.Label>
            <div className="d-flex align-items-center">
                <Badge bg={ip === 'Disconnected' ? 'danger' : 'success'} className="w-100 p-2">
                    {ip}
                </Badge>
            </div>
          </Form.Group>
          <Button variant="success" className="w-100" type="submit" disabled={isLoading}>
            Disconnect
          </Button>
        </Form>
      )}
    </div>
  );
}

export default Connection;
