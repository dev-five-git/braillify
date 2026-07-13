type Props = {
  dots: ReadonlySet<number> | number[]
  size?: number
  color?: string
}

export function BrailleCellIcon({
  dots,
  size = 22,
  color = 'currentColor',
}: Props) {
  const set = dots instanceof Set ? dots : new Set(dots)
  const r = size * 0.11
  const cx = [size * 0.3, size * 0.7]
  const cy = [size * 0.22, size * 0.5, size * 0.78]
  const positions: Array<[number, number, number]> = [
    [1, cx[0], cy[0]],
    [2, cx[0], cy[1]],
    [3, cx[0], cy[2]],
    [4, cx[1], cy[0]],
    [5, cx[1], cy[1]],
    [6, cx[1], cy[2]],
  ]

  return (
    <svg aria-hidden height={size} viewBox={`0 0 ${size} ${size}`} width={size}>
      {positions.map(([n, x, y]) => (
        <circle
          key={n}
          cx={x}
          cy={y}
          fill={set.has(n) ? color : 'transparent'}
          opacity={set.has(n) ? 1 : 0.35}
          r={r}
          stroke={color}
          strokeWidth={set.has(n) ? 0 : 1.2}
        />
      ))}
    </svg>
  )
}
