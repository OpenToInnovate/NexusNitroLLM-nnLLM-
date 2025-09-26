#!/usr/bin/env node
/**
 * # Chart Generator for Benchmark Reports
 * 
 * Generates simple ASCII charts and data exports for benchmark visualization.
 * Can be extended to generate PNG charts using libraries like Chart.js or D3.
 */

import { promises as fs } from 'node:fs';
import { execSync } from 'node:child_process';

const RESULTS_FILE = process.argv[2] || 'benchmark_results.json';
const OUTPUT_DIR = process.argv[3] || 'reports/charts';

class ChartGenerator {
    constructor() {
        this.results = null;
        this.metadata = null;
    }

    async loadResults() {
        try {
            const data = await fs.readFile(RESULTS_FILE, 'utf8');
            const parsed = JSON.parse(data);
            this.results = parsed.results;
            this.metadata = parsed.metadata;
            console.log(`📊 Loaded ${this.results.length} results for chart generation`);
        } catch (error) {
            throw new Error(`Failed to load results: ${error.message}`);
        }
    }

    generateThroughputChart() {
        // Group by language and concurrency
        const data = {};
        this.results.forEach(result => {
            if (!data[result.lang]) {
                data[result.lang] = {};
            }
            data[result.lang][result.concurrency] = result.throughput_rps;
        });

        const concurrencyLevels = [1, 8, 32, 128];
        const languages = Object.keys(data);

        let chart = 'Throughput vs Concurrency\n';
        chart += '='.repeat(50) + '\n';
        chart += 'Concurrency | ' + languages.map(lang => lang.padEnd(12)).join(' | ') + '\n';
        chart += '-'.repeat(50 + languages.length * 15) + '\n';

        concurrencyLevels.forEach(concurrency => {
            let row = `${concurrency.toString().padEnd(11)} | `;
            languages.forEach(lang => {
                const value = data[lang][concurrency] || 0;
                row += `${value.toFixed(0).padEnd(12)} | `;
            });
            chart += row + '\n';
        });

        return chart;
    }

    generateLatencyChart() {
        // Group by language and concurrency, focus on p99
        const data = {};
        this.results.forEach(result => {
            if (!data[result.lang]) {
                data[result.lang] = {};
            }
            data[result.lang][result.concurrency] = result.latency_ms.p99;
        });

        const concurrencyLevels = [1, 8, 32, 128];
        const languages = Object.keys(data);

        let chart = 'P99 Latency vs Concurrency (ms)\n';
        chart += '='.repeat(50) + '\n';
        chart += 'Concurrency | ' + languages.map(lang => lang.padEnd(12)).join(' | ') + '\n';
        chart += '-'.repeat(50 + languages.length * 15) + '\n';

        concurrencyLevels.forEach(concurrency => {
            let row = `${concurrency.toString().padEnd(11)} | `;
            languages.forEach(lang => {
                const value = data[lang][concurrency] || 0;
                row += `${value.toFixed(1).padEnd(12)} | `;
            });
            chart += row + '\n';
        });

        return chart;
    }

    generateErrorRateChart() {
        // Group by language and concurrency
        const data = {};
        this.results.forEach(result => {
            if (!data[result.lang]) {
                data[result.lang] = {};
            }
            const errorRate = (result.errors.total / result.reqs) * 100;
            data[result.lang][result.concurrency] = errorRate;
        });

        const concurrencyLevels = [1, 8, 32, 128];
        const languages = Object.keys(data);

        let chart = 'Error Rate vs Concurrency (%)\n';
        chart += '='.repeat(50) + '\n';
        chart += 'Concurrency | ' + languages.map(lang => lang.padEnd(12)).join(' | ') + '\n';
        chart += '-'.repeat(50 + languages.length * 15) + '\n';

        concurrencyLevels.forEach(concurrency => {
            let row = `${concurrency.toString().padEnd(11)} | `;
            languages.forEach(lang => {
                const value = data[lang][concurrency] || 0;
                row += `${value.toFixed(2).padEnd(12)} | `;
            });
            chart += row + '\n';
        });

        return chart;
    }

    generateCsvData() {
        const csvHeaders = 'language,route,concurrency,payload_size,throughput_rps,p50_ms,p95_ms,p99_ms,errors_total,success_rate,reqs,duration_s\n';
        
        const csvRows = this.results.map(result => {
            return [
                result.lang,
                result.route,
                result.concurrency,
                result.payload_size || 'S',
                result.throughput_rps.toFixed(2),
                result.latency_ms.p50.toFixed(3),
                result.latency_ms.p95.toFixed(3),
                result.latency_ms.p99.toFixed(3),
                result.errors.total,
                result.success_rate.toFixed(2),
                result.reqs,
                this.metadata?.benchmark_seconds || 60
            ].join(',');
        }).join('\n');

        return csvHeaders + csvRows;
    }

    generateJsonData() {
        // Structured data for external charting tools
        const chartData = {
            metadata: this.metadata,
            charts: {
                throughput_vs_concurrency: this.extractThroughputData(),
                latency_vs_concurrency: this.extractLatencyData(),
                error_rate_vs_concurrency: this.extractErrorRateData()
            },
            summary: this.generateSummaryStats()
        };

        return JSON.stringify(chartData, null, 2);
    }

    extractThroughputData() {
        const data = { series: [], categories: [1, 8, 32, 128] };
        const langGroups = {};

        this.results.forEach(result => {
            if (!langGroups[result.lang]) {
                langGroups[result.lang] = {};
            }
            langGroups[result.lang][result.concurrency] = result.throughput_rps;
        });

        Object.entries(langGroups).forEach(([lang, values]) => {
            data.series.push({
                name: lang,
                data: data.categories.map(c => values[c] || 0)
            });
        });

        return data;
    }

    extractLatencyData() {
        const data = { series: [], categories: [1, 8, 32, 128] };
        const langGroups = {};

        this.results.forEach(result => {
            if (!langGroups[result.lang]) {
                langGroups[result.lang] = {};
            }
            langGroups[result.lang][result.concurrency] = result.latency_ms.p99;
        });

        Object.entries(langGroups).forEach(([lang, values]) => {
            data.series.push({
                name: lang,
                data: data.categories.map(c => values[c] || 0)
            });
        });

        return data;
    }

    extractErrorRateData() {
        const data = { series: [], categories: [1, 8, 32, 128] };
        const langGroups = {};

        this.results.forEach(result => {
            if (!langGroups[result.lang]) {
                langGroups[result.lang] = {};
            }
            const errorRate = (result.errors.total / result.reqs) * 100;
            langGroups[result.lang][result.concurrency] = errorRate;
        });

        Object.entries(langGroups).forEach(([lang, values]) => {
            data.series.push({
                name: lang,
                data: data.categories.map(c => values[c] || 0)
            });
        });

        return data;
    }

    generateSummaryStats() {
        const langGroups = {};
        this.results.forEach(result => {
            if (!langGroups[result.lang]) {
                langGroups[result.lang] = [];
            }
            langGroups[result.lang].push(result);
        });

        const summary = {};
        Object.entries(langGroups).forEach(([lang, results]) => {
            const highConcurrency = results.filter(r => r.concurrency === 128);
            if (highConcurrency.length > 0) {
                const avg = this.averageResults(highConcurrency);
                summary[lang] = {
                    peak_throughput: Math.max(...results.map(r => r.throughput_rps)),
                    avg_throughput_c128: avg.throughput_rps,
                    best_p99: Math.min(...results.map(r => r.latency_ms.p99)),
                    avg_p99_c128: avg.latency_ms.p99,
                    total_requests: results.reduce((sum, r) => sum + r.reqs, 0),
                    avg_success_rate: avg.success_rate
                };
            }
        });

        return summary;
    }

    averageResults(results) {
        const avg = {
            throughput_rps: 0,
            latency_ms: { p50: 0, p95: 0, p99: 0 },
            success_rate: 0
        };

        results.forEach(result => {
            avg.throughput_rps += result.throughput_rps;
            avg.latency_ms.p50 += result.latency_ms.p50;
            avg.latency_ms.p95 += result.latency_ms.p95;
            avg.latency_ms.p99 += result.latency_ms.p99;
            avg.success_rate += result.success_rate;
        });

        const count = results.length;
        avg.throughput_rps /= count;
        avg.latency_ms.p50 /= count;
        avg.latency_ms.p95 /= count;
        avg.latency_ms.p99 /= count;
        avg.success_rate /= count;

        return avg;
    }

    async generateCharts() {
        await fs.mkdir(OUTPUT_DIR, { recursive: true });

        const throughputChart = this.generateThroughputChart();
        const latencyChart = this.generateLatencyChart();
        const errorChart = this.generateErrorRateChart();
        const csvData = this.generateCsvData();
        const jsonData = this.generateJsonData();

        // Save ASCII charts
        await fs.writeFile(path.join(OUTPUT_DIR, 'throughput_chart.txt'), throughputChart);
        await fs.writeFile(path.join(OUTPUT_DIR, 'latency_chart.txt'), latencyChart);
        await fs.writeFile(path.join(OUTPUT_DIR, 'error_rate_chart.txt'), errorChart);

        // Save data exports
        await fs.writeFile(path.join(OUTPUT_DIR, 'benchmark_data.csv'), csvData);
        await fs.writeFile(path.join(OUTPUT_DIR, 'chart_data.json'), jsonData);

        // Generate combined report
        const combinedReport = `# Benchmark Charts\n\n## Throughput vs Concurrency\n\`\`\`\n${throughputChart}\n\`\`\`\n\n## P99 Latency vs Concurrency\n\`\`\`\n${latencyChart}\n\`\`\`\n\n## Error Rate vs Concurrency\n\`\`\`\n${errorChart}\n\`\`\`\n\n## Data Exports\n- [CSV Data](benchmark_data.csv)\n- [JSON Data](chart_data.json)\n`;

        await fs.writeFile(path.join(OUTPUT_DIR, 'charts_report.md'), combinedReport);

        console.log(`📊 Charts generated in: ${OUTPUT_DIR}`);
        console.log(`📄 Combined report: ${path.join(OUTPUT_DIR, 'charts_report.md')}`);
        console.log(`📈 Data exports: CSV and JSON formats available`);

        return OUTPUT_DIR;
    }
}

async function main() {
    try {
        console.log('📊 Generating benchmark charts...');
        
        const generator = new ChartGenerator();
        await generator.loadResults();
        await generator.generateCharts();
        
        console.log('\n✅ Chart generation complete!');
        
    } catch (error) {
        console.error(`❌ Chart generation failed: ${error.message}`);
        process.exit(1);
    }
}

if (import.meta.url === `file://${process.argv[1]}`) {
    main().catch(console.error);
}

