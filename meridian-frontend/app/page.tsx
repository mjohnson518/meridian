'use client';

export default function HomePage() {
  return (
    <div style={{ minHeight: '100vh', backgroundColor: '#FFFFFF', color: '#000000' }}>
      {/* Hero Section */}
      <section style={{
        minHeight: '90vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        position: 'relative'
      }}>
        <div style={{
          textAlign: 'center',
          padding: '0 2rem',
          maxWidth: '1200px'
        }}>
          <div style={{
            display: 'inline-block',
            padding: '0.5rem 1.5rem',
            backgroundColor: 'rgba(255,255,255,0.2)',
            borderRadius: '9999px',
            marginBottom: '2rem'
          }}>
            <span style={{ color: '#FFFFFF', fontSize: '0.875rem', fontWeight: '600' }}>
              Live on Sepolia Testnet ✨
            </span>
          </div>
          
          <h1 style={{
            fontSize: 'clamp(3rem, 10vw, 6rem)',
            fontWeight: '800',
            color: '#FFFFFF',
            marginBottom: '1.5rem',
            lineHeight: '1.1'
          }}>
            Banking Infrastructure<br />
            <span style={{ 
              background: 'linear-gradient(to right, #FFD700, #FFA500)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent'
            }}>
              for the Future
            </span>
          </h1>
          
          <p style={{
            fontSize: '1.5rem',
            color: 'rgba(255,255,255,0.9)',
            maxWidth: '800px',
            margin: '0 auto 3rem'
          }}>
            Multi-currency stablecoins backed by sovereign bonds. Built for global finance with enterprise-grade compliance.
          </p>
          
          <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center', flexWrap: 'wrap' }}>
            <a href="/reserves" style={{
              padding: '1rem 2rem',
              fontSize: '1.125rem',
              fontWeight: '600',
              backgroundColor: '#FFFFFF',
              color: '#764ba2',
              border: 'none',
              borderRadius: '0.5rem',
              cursor: 'pointer',
              textDecoration: 'none',
              display: 'inline-block'
            }}>
              View Live Demo →
            </a>
            <a href="/docs" style={{
              padding: '1rem 2rem',
              fontSize: '1.125rem',
              fontWeight: '600',
              backgroundColor: 'transparent',
              color: '#FFFFFF',
              border: '2px solid #FFFFFF',
              borderRadius: '0.5rem',
              cursor: 'pointer',
              textDecoration: 'none',
              display: 'inline-block'
            }}>
              Documentation
            </a>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section style={{
        padding: '5rem 2rem',
        backgroundColor: '#F9FAFB'
      }}>
        <div style={{
          maxWidth: '1200px',
          margin: '0 auto',
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
          gap: '2rem'
        }}>
          {[
            { label: 'Total Value Locked', value: '$10,042,250', color: '#059669' },
            { label: 'Reserve Ratio', value: '100.42%', color: '#3B82F6' },
            { label: 'Active Currencies', value: '4', color: '#8B5CF6' },
            { label: 'Transactions Today', value: '1,247', color: '#F59E0B' }
          ].map((stat, i) => (
            <div key={i} style={{
              backgroundColor: '#FFFFFF',
              padding: '2rem',
              borderRadius: '1rem',
              boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
              transition: 'transform 0.2s',
              cursor: 'pointer'
            }}
            onMouseEnter={(e) => e.currentTarget.style.transform = 'translateY(-4px)'}
            onMouseLeave={(e) => e.currentTarget.style.transform = 'translateY(0)'}>
              <p style={{ color: '#6B7280', fontSize: '0.875rem', marginBottom: '0.5rem' }}>
                {stat.label}
              </p>
              <p style={{ fontSize: '2rem', fontWeight: '700', color: stat.color }}>
                {stat.value}
              </p>
            </div>
          ))}
        </div>
      </section>

      {/* Features Section */}
      <section style={{ padding: '5rem 2rem' }}>
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
          <h2 style={{
            fontSize: '3rem',
            fontWeight: '700',
            textAlign: 'center',
            marginBottom: '3rem'
          }}>
            Enterprise-Grade Infrastructure
          </h2>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(350px, 1fr))',
            gap: '2rem'
          }}>
            {[
              { title: 'Multi-Currency Support', desc: 'Deploy stablecoins for EUR, GBP, JPY, and emerging markets', icon: '🌍' },
              { title: 'x402 Agent Payments', desc: 'Built for the agentic economy with native AI payment protocols', icon: '🤖' },
              { title: 'Regulatory Compliant', desc: 'GENIUS Act, MiCA, and AML/KYC requirements built-in', icon: '✅' }
            ].map((feature, i) => (
              <div key={i} style={{
                padding: '2rem',
                borderRadius: '1rem',
                border: '1px solid #E5E7EB',
                transition: 'all 0.2s'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.boxShadow = '0 10px 30px rgba(0,0,0,0.1)';
                e.currentTarget.style.borderColor = '#764ba2';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.boxShadow = 'none';
                e.currentTarget.style.borderColor = '#E5E7EB';
              }}>
                <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>{feature.icon}</div>
                <h3 style={{ fontSize: '1.5rem', fontWeight: '600', marginBottom: '1rem' }}>
                  {feature.title}
                </h3>
                <p style={{ color: '#6B7280', lineHeight: '1.6' }}>
                  {feature.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
