import React from 'react';

interface JsonBlockProps {
  value: string;
  error?: boolean;
}

const containerStyle: React.CSSProperties = {
  background: '#0d0d0d',
  border: '1px solid #1e1e1e',
  borderRadius: '8px',
  padding: '0.75rem 0',
  fontFamily: "'SF Mono','Fira Code',monospace",
  fontSize: '0.75rem',
  overflowX: 'auto',
};

export default function JsonBlock({ value, error = false }: JsonBlockProps) {
  const lines = value.split('\n');
  const lineCount = lines.length;
  const numWidth = String(lineCount).length;

  return (
    <div style={containerStyle}>
      {lines.map((line, i) => (
        <div
          key={i}
          style={{ display: 'flex', gap: '0.75rem', paddingInline: '0.75rem', lineHeight: 1.6 }}
        >
          <span
            style={{
              color: '#3a3a3a',
              minWidth: `${numWidth}ch`,
              textAlign: 'right',
              userSelect: 'none',
              flexShrink: 0,
            }}
          >
            {i + 1}
          </span>
          <span
            style={{
              color: error ? '#ff6666' : '#9fff7e',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-all',
              flex: 1,
            }}
          >
            {line || '\u00a0'}
          </span>
        </div>
      ))}
    </div>
  );
}
