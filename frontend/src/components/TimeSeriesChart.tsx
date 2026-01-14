import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'
import './TimeSeriesChart.css'

interface TimeSeriesChartProps {
  history: Array<{ time: string; spread: number | null; midPrice: number | null }>
}

const TimeSeriesChart: React.FC<TimeSeriesChartProps> = ({ history }) => {
  // Transform data for Recharts
  const chartData = history.map((item, idx) => ({
    time: item.time,
    spread: item.spread,
    midPrice: item.midPrice,
    index: idx,
  }))

  if (history.length === 0) {
    return (
      <div className="timeseries-chart">
        <h2>Price & Spread History</h2>
        <div className="chart-loading">Waiting for data...</div>
      </div>
    )
  }

  return (
    <div className="timeseries-chart">
      <h2>Price & Spread History</h2>
      <div className="chart-container">
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="rgba(255, 255, 255, 0.1)" />
            <XAxis
              dataKey="time"
              stroke="rgba(255, 255, 255, 0.6)"
              style={{ fontSize: '12px' }}
              interval="preserveStartEnd"
            />
            <YAxis
              yAxisId="left"
              stroke="rgba(255, 255, 255, 0.6)"
              style={{ fontSize: '12px' }}
              label={{ value: 'Price', angle: -90, position: 'insideLeft', fill: 'rgba(255, 255, 255, 0.8)' }}
            />
            <YAxis
              yAxisId="right"
              orientation="right"
              stroke="rgba(255, 255, 255, 0.6)"
              style={{ fontSize: '12px' }}
              label={{ value: 'Spread', angle: 90, position: 'insideRight', fill: 'rgba(255, 255, 255, 0.8)' }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'rgba(10, 14, 39, 0.95)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                borderRadius: '8px',
                color: '#ffffff',
              }}
            />
            <Legend
              wrapperStyle={{ color: '#ffffff' }}
              iconType="line"
            />
            <Line
              yAxisId="left"
              type="monotone"
              dataKey="midPrice"
              stroke="#ffd700"
              strokeWidth={2}
              dot={false}
              name="Mid Price"
              isAnimationActive={false}
            />
            <Line
              yAxisId="right"
              type="monotone"
              dataKey="spread"
              stroke="#ff9800"
              strokeWidth={2}
              dot={false}
              name="Spread"
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}

export default TimeSeriesChart
