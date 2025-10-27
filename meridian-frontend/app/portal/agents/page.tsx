'use client';

import { useState, useEffect, useRef } from 'react';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { SacredCard } from '@/components/sacred/Card';
import { SacredButton } from '@/components/sacred/Button';
import { Heading, MonoDisplay, Label } from '@/components/sacred/Typography';
import { formatTimestamp, formatAddress } from '@/lib/utils';

interface Agent {
  agent_id: string;
  agent_name: string;
  wallet_address: string;
  spending_limit_daily: string;
  spending_limit_transaction: string;
  daily_spent: string;
  is_active: boolean;
  created_at: string;
}

interface AgentTransaction {
  id: number;
  currency: string;
  amount: string;
  recipient: string;
  status: string;
  transaction_hash: string | null;
  created_at: string;
}

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1';

export default function AgentsPage() {
  return (
    <ProtectedRoute>
      <AgentConsole />
    </ProtectedRoute>
  );
}

function AgentConsole() {
  const { user } = useAuth();
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [transactions, setTransactions] = useState<AgentTransaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [commandInput, setCommandInput] = useState('');
  const [terminalLog, setTerminalLog] = useState<string[]>([
    '> Meridian x402 Agent Terminal v1.0.0',
    '> Type "help" for available commands',
    ''
  ]);
  
  const terminalRef = useRef<HTMLDivElement>(null);

  // Load agents
  useEffect(() => {
    if (user?.id) {
      loadAgents();
    }
  }, [user]);

  const loadAgents = async () => {
    try {
      const response = await fetch(`${API_BASE_URL}/agents/list/${user?.id}`);
      if (response.ok) {
        const data = await response.json();
        setAgents(data.agents || []);
        if (data.agents && data.agents.length > 0 && !selectedAgent) {
          setSelectedAgent(data.agents[0]);
          loadTransactions(data.agents[0].agent_id);
        }
      }
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const loadTransactions = async (agentId: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/agents/transactions/${agentId}`);
      if (response.ok) {
        const data = await response.json();
        setTransactions(data.transactions || []);
      }
    } catch (error) {
      console.error('Failed to load transactions:', error);
    }
  };

  const addLog = (message: string) => {
    setTerminalLog(prev => [...prev, message]);
    setTimeout(() => {
      if (terminalRef.current) {
        terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
      }
    }, 100);
  };

  const handleCommand = async (cmd: string) => {
    const trimmedCmd = cmd.trim();
    if (!trimmedCmd) return;

    addLog(`> ${trimmedCmd}`);
    setCommandInput('');

    const parts = trimmedCmd.split(' ');
    const command = parts[0].toLowerCase();

    switch (command) {
      case 'help':
        addLog('Available commands:');
        addLog('  create <name> <daily_limit> <tx_limit> - Create new agent');
        addLog('  pay <recipient> <amount> <currency> - Send payment');
        addLog('  list - List all agents');
        addLog('  select <agent_id> - Select agent');
        addLog('  balance - Show current agent balance');
        addLog('  history - Show transaction history');
        addLog('  clear - Clear terminal');
        break;

      case 'create':
        if (parts.length < 4) {
          addLog('Error: Usage: create <name> <daily_limit> <tx_limit>');
          break;
        }
        await createAgent(parts[1], parts[2], parts[3]);
        break;

      case 'pay':
        if (parts.length < 4) {
          addLog('Error: Usage: pay <recipient> <amount> <currency>');
          break;
        }
        await executePay(parts[1], parts[2], parts[3]);
        break;

      case 'list':
        listAgents();
        break;

      case 'select':
        if (parts.length < 2) {
          addLog('Error: Usage: select <agent_id>');
          break;
        }
        selectAgent(parts[1]);
        break;

      case 'balance':
        showBalance();
        break;

      case 'history':
        showHistory();
        break;

      case 'clear':
        setTerminalLog(['> Terminal cleared', '']);
        break;

      default:
        addLog(`Unknown command: ${command}. Type "help" for available commands.`);
    }
  };

  const createAgent = async (name: string, dailyLimit: string, txLimit: string) => {
    setLoading(true);
    try {
      const response = await fetch(`${API_BASE_URL}/agents/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: parseInt(user?.id || '0'),
          agent_name: name,
          spending_limit_daily: dailyLimit,
          spending_limit_transaction: txLimit,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        addLog(`✓ Agent created successfully`);
        addLog(`  Agent ID: ${data.agent_id}`);
        addLog(`  API Key: ${data.api_key}`);
        addLog(`  Wallet: ${data.wallet_address}`);
        addLog(`  Daily Limit: $${dailyLimit}`);
        addLog(`  TX Limit: $${txLimit}`);
        addLog('');
        addLog('⚠️  Save the API key securely - it will not be shown again!');
        await loadAgents();
      } else {
        const error = await response.text();
        addLog(`✗ Failed to create agent: ${error}`);
      }
    } catch (error) {
      addLog(`✗ Error: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const executePay = async (recipient: string, amount: string, currency: string) => {
    if (!selectedAgent) {
      addLog('✗ No agent selected. Use "select <agent_id>" first.');
      return;
    }

    addLog(`Processing payment: ${amount} ${currency} → ${recipient}...`);
    setLoading(true);

    try {
      // Note: In production, this would use the agent's API key
      const response = await fetch(`${API_BASE_URL}/agents/pay`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          agent_id: selectedAgent.agent_id,
          api_key: 'stored_securely', // Would be retrieved from secure storage
          recipient,
          amount,
          currency,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        addLog(`✓ Payment executed`);
        addLog(`  TX Hash: ${data.transaction_hash}`);
        addLog(`  Status: ${data.status}`);
        await loadTransactions(selectedAgent.agent_id);
      } else {
        const error = await response.text();
        addLog(`✗ Payment failed: ${error}`);
      }
    } catch (error) {
      addLog(`✗ Error: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const listAgents = () => {
    if (agents.length === 0) {
      addLog('No agents configured. Create one with "create <name> <daily_limit> <tx_limit>"');
      return;
    }

    addLog(`Total agents: ${agents.length}`);
    agents.forEach((agent, i) => {
      addLog(`${i + 1}. ${agent.agent_name} (${agent.agent_id})`);
      addLog(`   Wallet: ${agent.wallet_address}`);
      addLog(`   Daily: $${agent.daily_spent}/$${agent.spending_limit_daily}`);
      addLog(`   Status: ${agent.is_active ? 'ACTIVE' : 'INACTIVE'}`);
    });
  };

  const selectAgent = (agentId: string) => {
    const agent = agents.find(a => a.agent_id.includes(agentId));
    if (agent) {
      setSelectedAgent(agent);
      loadTransactions(agent.agent_id);
      addLog(`✓ Selected agent: ${agent.agent_name}`);
    } else {
      addLog(`✗ Agent not found: ${agentId}`);
    }
  };

  const showBalance = () => {
    if (!selectedAgent) {
      addLog('✗ No agent selected');
      return;
    }

    addLog(`Agent: ${selectedAgent.agent_name}`);
    addLog(`Daily Spent: $${selectedAgent.daily_spent} / $${selectedAgent.spending_limit_daily}`);
    const remaining = parseFloat(selectedAgent.spending_limit_daily) - parseFloat(selectedAgent.daily_spent);
    addLog(`Remaining Today: $${remaining.toFixed(2)}`);
  };

  const showHistory = () => {
    if (transactions.length === 0) {
      addLog('No transactions yet');
      return;
    }

    addLog(`Recent transactions (${transactions.length}):`);
    transactions.slice(0, 10).forEach((tx, i) => {
      addLog(`${i + 1}. ${tx.amount} ${tx.currency} → ${tx.recipient.slice(0, 10)}...`);
      addLog(`   Status: ${tx.status}${tx.transaction_hash ? ` (${tx.transaction_hash.slice(0, 10)}...)` : ''}`);
    });
  };

  return (
    <div className="min-h-screen bg-sacred-black text-sacred-white">
      {/* Header */}
      <header className="border-b border-sacred-gray-700">
        <div className="sacred-container">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <a href="/portal/dashboard" className="font-mono text-lg font-medium text-sacred-white">
                MERIDIAN
              </a>
              <span className="text-sm font-mono uppercase tracking-wider text-sacred-gray-400">
                x402 Agent Console
              </span>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-xs font-mono uppercase text-sacred-gray-400">
                {user?.organization}
              </div>
            </div>
          </nav>
        </div>
      </header>

      {/* Main Content */}
      <div className="sacred-container py-8">
        <div className="grid grid-cols-12 gap-6">
          {/* Terminal Panel */}
          <div className="col-span-12 lg:col-span-8">
            <div className="bg-sacred-black border border-sacred-gray-700 rounded">
              {/* Terminal Header */}
              <div className="flex items-center justify-between px-4 py-2 border-b border-sacred-gray-700">
                <div className="flex items-center space-x-2">
                  <div className="w-3 h-3 rounded-full bg-red-500"></div>
                  <div className="w-3 h-3 rounded-full bg-amber-500"></div>
                  <div className="w-3 h-3 rounded-full bg-emerald-500"></div>
                  <span className="ml-4 text-xs font-mono text-sacred-gray-400">
                    agent@meridian:~$
                  </span>
                </div>
                {selectedAgent && (
                  <span className="text-xs font-mono text-emerald-400">
                    ● {selectedAgent.agent_name}
                  </span>
                )}
              </div>

              {/* Terminal Output */}
              <div
                ref={terminalRef}
                className="h-96 overflow-y-auto p-4 font-mono text-sm text-sacred-white"
                style={{ fontFamily: 'IBM Plex Mono, monospace' }}
              >
                {terminalLog.map((line, i) => (
                  <div key={i} className={line.startsWith('>') ? 'text-emerald-400' : 'text-sacred-gray-300'}>
                    {line}
                  </div>
                ))}
              </div>

              {/* Terminal Input */}
              <div className="border-t border-sacred-gray-700 p-4">
                <div className="flex items-center space-x-2">
                  <span className="text-emerald-400 font-mono">$</span>
                  <input
                    type="text"
                    value={commandInput}
                    onChange={(e) => setCommandInput(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        handleCommand(commandInput);
                      }
                    }}
                    className="flex-1 bg-transparent border-none outline-none font-mono text-sm text-sacred-white"
                    placeholder="Enter command..."
                    disabled={loading}
                    autoFocus
                  />
                  {loading && (
                    <span className="text-xs text-sacred-gray-500 font-mono">Processing...</span>
                  )}
                </div>
              </div>
            </div>

            {/* Quick Actions */}
            <div className="mt-4 flex flex-wrap gap-2">
              <button
                onClick={() => handleCommand('help')}
                className="px-3 py-1 bg-sacred-gray-800 text-sacred-white border border-sacred-gray-700 rounded font-mono text-xs hover:bg-sacred-gray-700"
              >
                help
              </button>
              <button
                onClick={() => handleCommand('list')}
                className="px-3 py-1 bg-sacred-gray-800 text-sacred-white border border-sacred-gray-700 rounded font-mono text-xs hover:bg-sacred-gray-700"
              >
                list agents
              </button>
              <button
                onClick={() => handleCommand('balance')}
                className="px-3 py-1 bg-sacred-gray-800 text-sacred-white border border-sacred-gray-700 rounded font-mono text-xs hover:bg-sacred-gray-700"
                disabled={!selectedAgent}
              >
                balance
              </button>
              <button
                onClick={() => handleCommand('history')}
                className="px-3 py-1 bg-sacred-gray-800 text-sacred-white border border-sacred-gray-700 rounded font-mono text-xs hover:bg-sacred-gray-700"
                disabled={!selectedAgent}
              >
                history
              </button>
              <button
                onClick={() => handleCommand('clear')}
                className="px-3 py-1 bg-sacred-gray-800 text-sacred-white border border-sacred-gray-700 rounded font-mono text-xs hover:bg-sacred-gray-700"
              >
                clear
              </button>
            </div>
          </div>

          {/* Info Panel */}
          <div className="col-span-12 lg:col-span-4 space-y-4">
            {selectedAgent ? (
              <>
                <div className="bg-sacred-gray-900 border border-sacred-gray-700 rounded p-4">
                  <h3 className="text-xs font-mono uppercase tracking-wider text-sacred-gray-400 mb-3">
                    Active Agent
                  </h3>
                  <div className="space-y-2 text-sm font-mono">
                    <div>
                      <div className="text-xs text-sacred-gray-500">Name</div>
                      <div className="text-emerald-400">{selectedAgent.agent_name}</div>
                    </div>
                    <div>
                      <div className="text-xs text-sacred-gray-500">ID</div>
                      <div className="text-xs break-all">{selectedAgent.agent_id}</div>
                    </div>
                    <div>
                      <div className="text-xs text-sacred-gray-500">Wallet</div>
                      <div className="text-xs break-all">{selectedAgent.wallet_address}</div>
                    </div>
                  </div>
                </div>

                <div className="bg-sacred-gray-900 border border-sacred-gray-700 rounded p-4">
                  <h3 className="text-xs font-mono uppercase tracking-wider text-sacred-gray-400 mb-3">
                    Spending Limits
                  </h3>
                  <div className="space-y-3">
                    <div>
                      <div className="flex justify-between text-xs mb-1">
                        <span className="text-sacred-gray-500">Daily</span>
                        <span className="text-sacred-white">
                          ${selectedAgent.daily_spent} / ${selectedAgent.spending_limit_daily}
                        </span>
                      </div>
                      <div className="h-2 bg-sacred-gray-800 rounded overflow-hidden">
                        <div
                          className="h-full bg-emerald-500"
                          style={{
                            width: `${(parseFloat(selectedAgent.daily_spent) / parseFloat(selectedAgent.spending_limit_daily)) * 100}%`
                          }}
                        />
                      </div>
                    </div>
                    <div>
                      <div className="text-xs text-sacred-gray-500">Per Transaction</div>
                      <div className="text-sm">${selectedAgent.spending_limit_transaction}</div>
                    </div>
                  </div>
                </div>

                <div className="bg-sacred-gray-900 border border-sacred-gray-700 rounded p-4">
                  <h3 className="text-xs font-mono uppercase tracking-wider text-sacred-gray-400 mb-3">
                    Quick Pay
                  </h3>
                  <div className="space-y-2">
                    <input
                      type="text"
                      placeholder="Recipient address"
                      className="w-full px-3 py-2 bg-sacred-gray-800 border border-sacred-gray-700 rounded font-mono text-xs text-sacred-white placeholder-sacred-gray-600"
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          const recipient = e.currentTarget.value;
                          const amount = (e.currentTarget.nextElementSibling as HTMLInputElement)?.value;
                          if (recipient && amount) {
                            handleCommand(`pay ${recipient} ${amount} EUR`);
                          }
                        }
                      }}
                    />
                    <input
                      type="number"
                      placeholder="Amount"
                      className="w-full px-3 py-2 bg-sacred-gray-800 border border-sacred-gray-700 rounded font-mono text-xs text-sacred-white placeholder-sacred-gray-600"
                      step="0.01"
                    />
                    <button
                      onClick={() => {
                        const inputs = document.querySelectorAll('input[placeholder="Recipient address"], input[placeholder="Amount"]') as NodeListOf<HTMLInputElement>;
                        const recipient = inputs[0]?.value;
                        const amount = inputs[1]?.value;
                        if (recipient && amount) {
                          handleCommand(`pay ${recipient} ${amount} EUR`);
                          inputs[0].value = '';
                          inputs[1].value = '';
                        }
                      }}
                      className="w-full px-3 py-2 bg-emerald-600 text-sacred-black font-mono text-xs uppercase tracking-wider rounded hover:bg-emerald-500"
                    >
                      Execute Payment →
                    </button>
                  </div>
                </div>
              </>
            ) : (
              <div className="bg-sacred-gray-900 border border-sacred-gray-700 rounded p-6 text-center">
                <p className="text-sm font-mono text-sacred-gray-400 mb-4">
                  No agent selected
                </p>
                <button
                  onClick={() => handleCommand('create MyAgent 10000 5000')}
                  className="px-4 py-2 bg-emerald-600 text-sacred-black font-mono text-xs uppercase tracking-wider rounded hover:bg-emerald-500"
                >
                  Create Agent
                </button>
              </div>
            )}

            {/* Recent Transactions */}
            {transactions.length > 0 && (
              <div className="bg-sacred-gray-900 border border-sacred-gray-700 rounded p-4">
                <h3 className="text-xs font-mono uppercase tracking-wider text-sacred-gray-400 mb-3">
                  Recent Transactions
                </h3>
                <div className="space-y-2 text-xs font-mono">
                  {transactions.slice(0, 5).map((tx) => (
                    <div key={tx.id} className="border-b border-sacred-gray-800 pb-2">
                      <div className="flex justify-between">
                        <span className="text-sacred-gray-400">{tx.currency}</span>
                        <span className="text-emerald-400">${tx.amount}</span>
                      </div>
                      <div className="text-sacred-gray-500 mt-1">
                        {tx.recipient.slice(0, 16)}...
                      </div>
                      <div className={`mt-1 ${
                        tx.status === 'COMPLETED' ? 'text-emerald-400' :
                        tx.status === 'PENDING' ? 'text-amber-400' :
                        'text-red-400'
                      }`}>
                        {tx.status}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Documentation */}
        <div className="mt-8 p-6 bg-sacred-gray-900 border border-sacred-gray-700 rounded">
          <Heading level={3} className="text-lg font-mono uppercase mb-4 text-sacred-white">
            x402 Agent Payment Protocol
          </Heading>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-sm">
            <div>
              <h4 className="text-xs font-mono uppercase text-emerald-400 mb-2">
                What is x402?
              </h4>
              <p className="text-sacred-gray-400 text-xs leading-relaxed">
                x402 enables AI agents to make autonomous payments using stablecoins.
                Each agent has its own wallet, spending limits, and transaction history.
              </p>
            </div>
            <div>
              <h4 className="text-xs font-mono uppercase text-emerald-400 mb-2">
                Use Cases
              </h4>
              <ul className="text-sacred-gray-400 text-xs space-y-1">
                <li>• AI agent marketplace purchases</li>
                <li>• Autonomous treasury management</li>
                <li>• Cross-border payments</li>
                <li>• Recurring subscriptions</li>
              </ul>
            </div>
            <div>
              <h4 className="text-xs font-mono uppercase text-emerald-400 mb-2">
                Security
              </h4>
              <ul className="text-sacred-gray-400 text-xs space-y-1">
                <li>• Daily spending limits</li>
                <li>• Per-transaction caps</li>
                <li>• API key authentication</li>
                <li>• Real-time monitoring</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

