'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { PortalHeader } from '@/components/portal/PortalHeader';
import { PortalCard } from '@/components/portal/PortalCard';
import { api, AgentRecord, AgentTransaction } from '@/lib/api/client';

export default function AgentsPage() {
  return (
    <ProtectedRoute>
      <AgentConsole />
    </ProtectedRoute>
  );
}

function AgentConsole() {
  const { user } = useAuth();
  const [agents, setAgents] = useState<AgentRecord[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<AgentRecord | null>(null);
  const [transactions, setTransactions] = useState<AgentTransaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [commandInput, setCommandInput] = useState('');
  const [terminalLog, setTerminalLog] = useState<string[]>([
    '> Meridian x402 Agent Terminal v1.0.0',
    '> Type "help" for available commands',
    '',
  ]);

  const terminalRef = useRef<HTMLDivElement>(null);

  const addLog = useCallback((message: string) => {
    setTerminalLog(prev => [...prev, message]);
    setTimeout(() => {
      if (terminalRef.current) {
        terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
      }
    }, 100);
  }, []);

  const loadTransactions = useCallback(async (agentId: string) => {
    try {
      const data = await api.getAgentTransactions(agentId);
      setTransactions(data.transactions ?? []);
    } catch (error) {
      console.error('Failed to load transactions:', error);
    }
  }, []);

  const loadAgents = useCallback(async () => {
    if (!user?.id) return;
    try {
      const data = await api.listAgents(user.id);
      const list = data.agents ?? [];
      setAgents(list);
      if (list.length > 0 && !selectedAgent) {
        setSelectedAgent(list[0]);
        loadTransactions(list[0].agent_id);
      }
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  }, [user?.id, selectedAgent, loadTransactions]);

  useEffect(() => {
    if (user?.id) {
      loadAgents();
    }
  }, [user?.id, loadAgents]);

  const createAgent = async (name: string, dailyLimit: string, txLimit: string) => {
    setLoading(true);
    try {
      const data = await api.createAgent({
        user_id: parseInt(user?.id ?? '0'),
        agent_name: name,
        spending_limit_daily: dailyLimit,
        spending_limit_transaction: txLimit,
      });
      const maskedKey = data.api_key
        ? `${data.api_key.slice(0, 8)}${'*'.repeat(16)}${data.api_key.slice(-4)}`
        : 'ERROR: No key returned';

      addLog('✓ Agent created successfully');
      addLog(`  Agent ID: ${data.agent_id}`);
      addLog(`  API Key: ${maskedKey}`);
      addLog(`  Wallet: ${data.wallet_address}`);
      addLog('');
      addLog('⚠  Copy full API key from response - masked in terminal for security!');

      if (data.api_key && navigator.clipboard) {
        navigator.clipboard.writeText(data.api_key).then(() => {
          addLog('📋 API key copied to clipboard (auto-clears in 30s)');
          setTimeout(() => navigator.clipboard.writeText('').catch(() => {}), 30000);
        }).catch(() => {
          addLog('⚠  Could not copy to clipboard - save key manually');
        });
      }

      await loadAgents();
    } catch (error) {
      addLog(`✗ Failed to create agent: ${error instanceof Error ? error.message : String(error)}`);
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
      const data = await api.payAgent({
        agent_id: selectedAgent.agent_id,
        api_key: 'stored_securely',
        recipient,
        amount,
        currency,
      });
      addLog('✓ Payment executed');
      addLog(`  TX Hash: ${data.transaction_hash}`);
      addLog(`  Status: ${data.status}`);
      await loadTransactions(selectedAgent.agent_id);
    } catch (error) {
      addLog(`✗ Payment failed: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setLoading(false);
    }
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
        addLog('  pay <recipient> <amount> <currency>    - Send payment');
        addLog('  list    - List all agents');
        addLog('  select  <agent_id> - Select agent');
        addLog('  balance - Show current agent balance');
        addLog('  history - Show transaction history');
        addLog('  clear   - Clear terminal');
        break;
      case 'create':
        if (parts.length < 4) { addLog('Error: Usage: create <name> <daily_limit> <tx_limit>'); break; }
        await createAgent(parts[1], parts[2], parts[3]);
        break;
      case 'pay':
        if (parts.length < 4) { addLog('Error: Usage: pay <recipient> <amount> <currency>'); break; }
        await executePay(parts[1], parts[2], parts[3]);
        break;
      case 'list':
        if (agents.length === 0) {
          addLog('No agents. Create one with: create <name> <daily_limit> <tx_limit>');
        } else {
          addLog(`Total agents: ${agents.length}`);
          agents.forEach((a, i) => {
            addLog(`${i + 1}. ${a.agent_name} (${a.agent_id})`);
            addLog(`   Daily: $${a.daily_spent}/$${a.spending_limit_daily} | ${a.is_active ? 'ACTIVE' : 'INACTIVE'}`);
          });
        }
        break;
      case 'select':
        if (parts.length < 2) { addLog('Error: Usage: select <agent_id>'); break; }
        const found = agents.find(a => a.agent_id.includes(parts[1]));
        if (found) {
          setSelectedAgent(found);
          loadTransactions(found.agent_id);
          addLog(`✓ Selected agent: ${found.agent_name}`);
        } else {
          addLog(`✗ Agent not found: ${parts[1]}`);
        }
        break;
      case 'balance':
        if (!selectedAgent) { addLog('✗ No agent selected'); break; }
        addLog(`Agent: ${selectedAgent.agent_name}`);
        addLog(`Daily Spent: $${selectedAgent.daily_spent} / $${selectedAgent.spending_limit_daily}`);
        addLog(`Remaining: $${(parseFloat(selectedAgent.spending_limit_daily) - parseFloat(selectedAgent.daily_spent)).toFixed(2)}`);
        break;
      case 'history':
        if (transactions.length === 0) { addLog('No transactions yet'); break; }
        addLog(`Recent transactions (${transactions.length}):`);
        transactions.slice(0, 10).forEach((tx, i) => {
          addLog(`${i + 1}. ${tx.amount} ${tx.currency} → ${tx.recipient.slice(0, 16)}...`);
          addLog(`   ${tx.status}${tx.transaction_hash ? ` (${tx.transaction_hash.slice(0, 12)}...)` : ''}`);
        });
        break;
      case 'clear':
        setTerminalLog(['> Terminal cleared', '']);
        break;
      default:
        addLog(`Unknown command: ${command}. Type "help" for available commands.`);
    }
  };

  const quickButtonClass =
    'px-3 py-1 bg-gray-800 text-white border border-gray-700 rounded font-mono text-xs hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed';

  return (
    <div className="min-h-screen dark:bg-gray-950">
      <PortalHeader currentPath="/portal/agents" />

      <div className="max-w-7xl mx-auto px-6 py-8">
        <div className="mb-6">
          <h1 className="text-3xl font-heading font-bold text-white mb-1">x402 Agent Console</h1>
          <p className="text-sm text-gray-500 font-mono">Autonomous payment agents for AI-driven treasury operations</p>
        </div>

        <div className="grid grid-cols-12 gap-6">
          {/* Terminal Panel */}
          <div className="col-span-12 lg:col-span-8">
            <div className="bg-gray-950 border border-gray-800 rounded-2xl overflow-hidden">
              {/* Terminal chrome */}
              <div className="flex items-center justify-between px-4 py-3 border-b border-gray-800 bg-gray-900/50">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-red-500/80" />
                  <div className="w-3 h-3 rounded-full bg-amber-500/80" />
                  <div className="w-3 h-3 rounded-full bg-emerald-500/80" />
                  <span className="ml-4 text-xs font-mono text-gray-500">agent@meridian:~$</span>
                </div>
                {selectedAgent && (
                  <span className="text-xs font-mono text-emerald-400">● {selectedAgent.agent_name}</span>
                )}
              </div>

              {/* Output */}
              <div
                ref={terminalRef}
                className="h-96 overflow-y-auto p-4 font-mono text-sm"
                style={{ fontFamily: "'JetBrains Mono', 'IBM Plex Mono', monospace" }}
              >
                {terminalLog.map((line, i) => (
                  <div
                    key={i}
                    className={
                      line.startsWith('>') ? 'text-emerald-400' :
                      line.startsWith('✗') ? 'text-red-400' :
                      line.startsWith('✓') ? 'text-emerald-300' :
                      line.startsWith('⚠') ? 'text-amber-400' :
                      'text-gray-300'
                    }
                  >
                    {line}
                  </div>
                ))}
              </div>

              {/* Input */}
              <div className="border-t border-gray-800 px-4 py-3">
                <div className="flex items-center gap-2">
                  <span className="text-emerald-400 font-mono text-sm">$</span>
                  <input
                    type="text"
                    value={commandInput}
                    onChange={(e) => setCommandInput(e.target.value)}
                    onKeyDown={(e) => { if (e.key === 'Enter') handleCommand(commandInput); }}
                    className="flex-1 bg-transparent border-none outline-none font-mono text-sm text-white placeholder-gray-600"
                    placeholder="Enter command..."
                    disabled={loading}
                    autoFocus
                    style={{ fontFamily: "'JetBrains Mono', monospace" }}
                  />
                  {loading && <span className="text-xs text-gray-500 font-mono">Processing...</span>}
                </div>
              </div>
            </div>

            {/* Quick actions */}
            <div className="mt-3 flex flex-wrap gap-2">
              {(['help', 'list', 'balance', 'history', 'clear'] as const).map(cmd => (
                <button
                  key={cmd}
                  onClick={() => handleCommand(cmd)}
                  disabled={loading || (['balance', 'history'].includes(cmd) && !selectedAgent)}
                  className={quickButtonClass}
                >
                  {cmd}
                </button>
              ))}
            </div>
          </div>

          {/* Info Panel */}
          <div className="col-span-12 lg:col-span-4 space-y-4">
            {selectedAgent ? (
              <>
                <PortalCard header="Active Agent" hoverEffect={false} padding="sm">
                  <div className="space-y-3 font-mono text-sm">
                    <div>
                      <p className="text-xs text-gray-500 mb-0.5">Name</p>
                      <p className="text-emerald-400">{selectedAgent.agent_name}</p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500 mb-0.5">Agent ID</p>
                      <p className="text-xs text-gray-300 break-all">{selectedAgent.agent_id}</p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500 mb-0.5">Wallet</p>
                      <p className="text-xs text-gray-300 break-all">{selectedAgent.wallet_address}</p>
                    </div>
                  </div>
                </PortalCard>

                <PortalCard header="Spending Limits" hoverEffect={false} padding="sm">
                  <div className="space-y-3">
                    <div>
                      <div className="flex justify-between font-mono text-xs mb-1.5">
                        <span className="text-gray-500">Daily</span>
                        <span className="text-white">${selectedAgent.daily_spent} / ${selectedAgent.spending_limit_daily}</span>
                      </div>
                      <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-emerald-500 rounded-full transition-all"
                          style={{
                            width: `${Math.min(100, (parseFloat(selectedAgent.daily_spent) / parseFloat(selectedAgent.spending_limit_daily)) * 100)}%`
                          }}
                        />
                      </div>
                    </div>
                    <div className="flex justify-between font-mono text-xs">
                      <span className="text-gray-500">Per Transaction</span>
                      <span className="text-white">${selectedAgent.spending_limit_transaction}</span>
                    </div>
                  </div>
                </PortalCard>
              </>
            ) : (
              <PortalCard hoverEffect={false} padding="md">
                <div className="text-center py-4">
                  <p className="text-sm font-mono text-gray-500 mb-4">No agent selected</p>
                  <button
                    onClick={() => handleCommand('create MyAgent 10000 5000')}
                    className="px-4 py-2 bg-emerald-600 text-white font-mono text-xs uppercase tracking-wider rounded-xl hover:bg-emerald-500 transition-colors"
                  >
                    Create Agent
                  </button>
                </div>
              </PortalCard>
            )}

            {transactions.length > 0 && (
              <PortalCard header="Recent Transactions" hoverEffect={false} padding="sm">
                <div className="space-y-2">
                  {transactions.slice(0, 5).map((tx) => (
                    <div key={tx.id} className="border-b border-white/5 pb-2 last:border-0 last:pb-0">
                      <div className="flex justify-between font-mono text-xs">
                        <span className="text-gray-400">{tx.currency}</span>
                        <span className="text-emerald-400">${tx.amount}</span>
                      </div>
                      <div className="text-gray-500 text-xs mt-0.5 font-mono">
                        {tx.recipient.slice(0, 18)}...
                      </div>
                      <div className={`text-xs font-mono mt-0.5 ${
                        tx.status === 'COMPLETED' ? 'text-emerald-400' :
                        tx.status === 'PENDING' ? 'text-amber-400' : 'text-red-400'
                      }`}>
                        {tx.status}
                      </div>
                    </div>
                  ))}
                </div>
              </PortalCard>
            )}
          </div>
        </div>

        {/* Documentation */}
        <PortalCard hoverEffect={false} header="x402 Agent Payment Protocol" className="mt-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-sm">
            <div>
              <h4 className="text-xs font-mono uppercase text-emerald-400 mb-2">What is x402?</h4>
              <p className="text-gray-400 text-xs leading-relaxed">
                x402 enables AI agents to make autonomous payments using stablecoins.
                Each agent has its own wallet, spending limits, and transaction history.
              </p>
            </div>
            <div>
              <h4 className="text-xs font-mono uppercase text-emerald-400 mb-2">Use Cases</h4>
              <ul className="text-gray-400 text-xs space-y-1">
                <li>• AI agent marketplace purchases</li>
                <li>• Autonomous treasury management</li>
                <li>• Cross-border payments</li>
                <li>• Recurring subscriptions</li>
              </ul>
            </div>
            <div>
              <h4 className="text-xs font-mono uppercase text-emerald-400 mb-2">Security</h4>
              <ul className="text-gray-400 text-xs space-y-1">
                <li>• Daily spending limits</li>
                <li>• Per-transaction caps</li>
                <li>• API key authentication</li>
                <li>• Real-time monitoring</li>
              </ul>
            </div>
          </div>
        </PortalCard>
      </div>
    </div>
  );
}
