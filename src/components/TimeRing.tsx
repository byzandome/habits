import { PieChart, Pie, Cell, ResponsiveContainer } from 'recharts';

interface Props {
  productiveSecs: number;
  idleSecs: number;
  lockedSecs: number;
}

function formatCenter(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${secs}s`;
}

const PRODUCTIVE_COLOR = '#22C55E';
const IDLE_COLOR = '#1E293B';
const LOCKED_COLOR = '#F59E0B';
const EMPTY_COLOR = '#1E293B';

export function TimeRing({ productiveSecs, idleSecs, lockedSecs }: Props) {
  const total = productiveSecs + idleSecs + lockedSecs;

  const data =
    total > 0
      ? [
          { name: 'Productive', value: productiveSecs, color: PRODUCTIVE_COLOR },
          { name: 'Idle',       value: idleSecs,       color: IDLE_COLOR },
          { name: 'Locked',     value: lockedSecs,     color: LOCKED_COLOR },
        ].filter((d) => d.value > 0)
      : [{ name: 'Empty', value: 1, color: EMPTY_COLOR }];

  return (
    <div style={{ position: 'relative', width: '100%', height: 300 }}>
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={95}
            outerRadius={135}
            startAngle={90}
            endAngle={-270}
            paddingAngle={total > 0 ? 2 : 0}
            dataKey="value"
            strokeWidth={0}
          >
            {data.map((entry, i) => (
              <Cell key={i} fill={entry.color} />
            ))}
          </Pie>
        </PieChart>
      </ResponsiveContainer>

      {/* Center label */}
      <div
        style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          textAlign: 'center',
          pointerEvents: 'none',
          userSelect: 'none',
        }}
      >
        <div style={{ fontSize: 38, fontWeight: 700, color: '#F8FAFC', lineHeight: 1 }}>
          {formatCenter(productiveSecs)}
        </div>
        <div style={{ fontSize: 13, color: '#64748B', marginTop: 6, letterSpacing: '0.05em', textTransform: 'uppercase' }}>
          productive
        </div>
        {total > 0 && (
          <div style={{ fontSize: 12, color: '#475569', marginTop: 4 }}>
            {Math.round((productiveSecs / total) * 100)}% of today
          </div>
        )}
      </div>
    </div>
  );
}
