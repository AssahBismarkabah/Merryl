import {
  BarController,
  BarElement,
  CategoryScale,
  Chart,
  LinearScale,
  Tooltip,
  type ChartConfiguration
} from "chart.js";
import { useEffect, useRef } from "react";
import { number, percent } from "../format";
import type { Sector } from "../types";

Chart.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip);

export function SectorScoreChart({ sectors }: { sectors: Sector[] }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const topSectors = sectors.slice(0, 11);
    const chart = new Chart(canvas, chartConfig(topSectors));

    return () => chart.destroy();
  }, [sectors]);

  return (
    <div className="overviewChartBlock">
      <div className="chartBlockHeader">
        <h3>Sector Score</h3>
        <span className="dataTag muted">Top 11</span>
      </div>
      <div className="chartCanvasWrap">
        <canvas ref={canvasRef} />
      </div>
    </div>
  );
}

function chartConfig(sectors: Sector[]): ChartConfiguration<"bar"> {
  return {
    type: "bar",
    data: {
      labels: sectors.map((sector) => sector.sector),
      datasets: [
        {
          label: "Score",
          data: sectors.map((sector) => sector.score),
          backgroundColor: "#4fc3f7",
          borderColor: "transparent",
          borderSkipped: false,
          borderWidth: 0,
          borderRadius: 3,
          barThickness: 14,
          maxBarThickness: 14
        }
      ]
    },
    options: {
      animation: false,
      indexAxis: "y",
      maintainAspectRatio: false,
      responsive: true,
      layout: {
        padding: { right: 24 }
      },
      plugins: {
        legend: {
          display: false
        },
        tooltip: {
          backgroundColor: "#1a1a1a",
          bodyColor: "#eeeeee",
          borderColor: "#444444",
          borderWidth: 1,
          cornerRadius: 6,
          displayColors: false,
          padding: 10,
          titleColor: "#eeeeee",
          callbacks: {
            label(context) {
              const sector = sectors[context.dataIndex];
              if (!sector) {
                return "";
              }
              return [
                `Score: ${number(sector.score)}`,
                `20D: ${percent(sector.return_20d)}`,
                `Vs SPY: ${percent(sector.relative_return_vs_spy)}`
              ];
            }
          }
        }
      },
      scales: {
        x: {
          max: 100,
          min: 0,
          grid: {
            color: "#252525"
          },
          border: {
            display: false
          },
          ticks: {
            color: "#666666",
            font: { size: 10 }
          }
        },
        y: {
          grid: {
            display: false
          },
          border: {
            display: false
          },
          ticks: {
            color: "#bbbbbb",
            font: { size: 11 },
            padding: 0
          }
        }
      }
    }
  };
}
