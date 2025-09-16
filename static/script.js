class SQLTraceApp {
    constructor() {
        this.queryInput = document.getElementById('queryInput');
        this.executeBtn = document.getElementById('executeBtn');
        this.errorContainer = document.getElementById('errorContainer');
        this.errorText = document.getElementById('errorText');
        this.planContainer = document.getElementById('planContainer');
        this.exportSection = document.getElementById('exportSection');
        this.advisorSection = document.getElementById('advisorSection');
        this.advisorContent = document.getElementById('advisorContent');
        
        this.historySection = document.getElementById('historySection');
        this.historyList = document.getElementById('historyList');
        this.clearHistoryBtn = document.getElementById('clearHistory');
        this.toggleComparisonBtn = document.getElementById('toggleComparison');
        this.comparisonSection = document.getElementById('comparisonSection');
        this.exitComparisonBtn = document.getElementById('exitComparison');
        this.comparisonResults = document.getElementById('comparisonResults');
        
        this.currentPlanData = null;
        this.currentAdvisorAnalysis = null;
        this.queryHistory = this.loadHistoryFromStorage();
        this.comparisonMode = false;
        this.selectedQueries = [];
        
        this.init();
    }

    init() {
        this.executeBtn.addEventListener('click', () => this.executeQuery());
        this.queryInput.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'Enter') {
                this.executeQuery();
            }
        });

        document.querySelectorAll('.example-query').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const query = e.target.getAttribute('data-query');
                this.queryInput.value = query;
                this.executeQuery();
            });
        });

        document.getElementById('exportJson').addEventListener('click', () => this.exportAsJson());
        document.getElementById('exportText').addEventListener('click', () => this.exportAsText());
        document.getElementById('exportHtml').addEventListener('click', () => this.exportAsHtml());
        document.getElementById('copyPlan').addEventListener('click', () => this.copyToClipboard());

        this.clearHistoryBtn.addEventListener('click', () => this.clearHistory());
        this.toggleComparisonBtn.addEventListener('click', () => this.toggleComparison());
        this.exitComparisonBtn.addEventListener('click', () => this.exitComparison());
        this.initializeTheme();
        this.renderHistory();
    }

    async executeQuery() {
        const query = this.queryInput.value.trim();
        
        if (!query) {
            this.showError('Please enter a SQL query');
            return;
        }

        this.setLoading(true);
        this.hideError();

        try {
            const response = await fetch('/api/explain', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ query })
            });

            const data = await response.json();

            if (data.error) {
                this.showError(data.error);
                this.showEmptyState();
            } else {
                this.renderPlan(data.plan);
                this.renderPerformanceMetrics(data.plan);
                this.renderAdvisorSuggestions(data.advisor_analysis);
                this.exportSection.style.display = 'block';
                
                this.addToHistory(query, data.plan, data.advisor_analysis);
            }
        } catch (error) {
            console.error('Error executing query:', error);
            this.showError('Failed to execute query. Please check your SQL syntax and try again.');
        } finally {
            this.executeBtn.disabled = false;
            this.executeBtn.textContent = 'Analyze Query';
        }
    }

    setLoading(loading) {
        this.executeBtn.disabled = loading;
        this.executeBtn.querySelector('.btn-text').textContent = loading ? 'Analyzing...' : 'Analyze Query';
        this.executeBtn.querySelector('.btn-spinner').style.display = loading ? 'inline-block' : 'none';
        
        const querySection = document.querySelector('.query-section');
        if (loading) {
            querySection.classList.add('loading');
        } else {
            querySection.classList.remove('loading');
        }
    }

    showError(message) {
        this.errorText.textContent = message;
        this.errorContainer.style.display = 'block';
    }

    hideError() {
        this.errorContainer.style.display = 'none';
    }

    showEmptyState() {
        this.planContainer.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon">📊</div>
                <p>Click "Analyze Query" to visualize the execution plan</p>
            </div>
        `;
    }

    renderPlan(planData) {
        if (!planData || !planData.nodes || planData.nodes.length === 0) {
            this.showEmptyState();
            return;
        }

        this.currentPlanData = planData;

        const performanceMetrics = this.renderPerformanceMetrics(planData);
        
        const planTree = document.createElement('div');
        planTree.className = 'plan-tree';
        
        planData.root_indices.forEach(rootIdx => {
            this.renderNode(planTree, planData.nodes, rootIdx, 0);
        });

        this.planContainer.innerHTML = performanceMetrics;
        this.planContainer.appendChild(planTree);
        
        this.exportSection.style.display = 'block';
    }

    renderNode(container, nodes, nodeIdx, level) {
        const node = nodes[nodeIdx];
        if (!node) return;

        const nodeElement = document.createElement('div');
        nodeElement.className = 'plan-node';
        nodeElement.setAttribute('data-level', level);
        nodeElement.setAttribute('data-node-idx', nodeIdx);

        const hasChildren = node.children && node.children.length > 0;
        
        if (hasChildren) {
            nodeElement.classList.add('expandable');
            if (node.expanded) {
                nodeElement.classList.add('expanded');
            }
        }

        const nodeContent = this.createNodeContent(node, level, hasChildren, node.expanded);
        nodeElement.innerHTML = nodeContent;

        if (hasChildren) {
            nodeElement.addEventListener('click', (e) => {
                e.stopPropagation();
                this.toggleNode(nodeElement, nodes, nodeIdx);
            });
        }

        container.appendChild(nodeElement);

        if (hasChildren && node.expanded) {
            node.children.forEach(childIdx => {
                this.renderNode(container, nodes, childIdx, level + 1);
            });
        }
    }

    createNodeContent(planNode, level, hasChildren, isExpanded) {
        const indent = '  '.repeat(level);
        const expandIcon = hasChildren ? (isExpanded ? '▼' : '▶') : '';
        
        const nodeType = planNode.node_type || 'Unknown';
        const relationName = planNode.relation_name || '';
        const alias = planNode.alias || '';
        
        let nodeTitle = `${indent}${expandIcon} <span class="plan-node-type">${nodeType}</span>`;
        if (relationName && relationName !== alias) {
            nodeTitle += ` on ${relationName}`;
        }
        if (alias) {
            nodeTitle += ` (${alias})`;
        }

        const details = [];
        
        if (planNode.total_cost !== undefined) {
            details.push(`<span class="plan-node-cost">Cost: ${planNode.startup_cost?.toFixed(2) || 0}..${planNode.total_cost.toFixed(2)}</span>`);
        }
        
        if (planNode.actual_total_time !== undefined) {
            details.push(`<span class="plan-node-time">Time: ${planNode.actual_startup_time?.toFixed(3) || 0}..${planNode.actual_total_time.toFixed(3)}ms</span>`);
        }
        
        if (planNode.actual_rows !== undefined) {
            details.push(`<span class="plan-node-rows">Actual Rows: ${planNode.actual_rows}</span>`);
        }

        if (planNode.extra && typeof planNode.extra === 'object') {
            if (planNode.extra['Index Cond']) {
                details.push(`Index Cond: ${planNode.extra['Index Cond']}`);
            }
            
            if (planNode.extra['Filter']) {
                details.push(`Filter: ${planNode.extra['Filter']}`);
            }
            
            if (planNode.extra['Hash Cond']) {
                details.push(`Hash Cond: ${planNode.extra['Hash Cond']}`);
            }
        }

        const detailsHtml = details.length > 0 
            ? `<div class="plan-node-details">${details.join(' • ')}</div>`
            : '';

        return nodeTitle + detailsHtml;
    }

    toggleNode(nodeElement, nodes, nodeIdx) {
        const node = nodes[nodeIdx];
        node.expanded = !node.expanded;
        
        const planData = {
            nodes: nodes,
            root_indices: this.getRootIndices(nodes)
        };
        this.renderPlan(planData);
    }

    getRootIndices(nodes) {
        // Find nodes that are not children of any other node
        const allChildren = new Set();
        nodes.forEach(node => {
            if (node.children) {
                node.children.forEach(childIdx => allChildren.add(childIdx));
            }
        });
        
        return nodes.map((_, idx) => idx).filter(idx => !allChildren.has(idx));
    }

    exportAsJson() {
        if (!this.currentPlanData) return;
        
        const dataStr = JSON.stringify(this.currentPlanData, null, 2);
        this.downloadFile(dataStr, 'execution-plan.json', 'application/json');
    }

    exportAsText() {
        if (!this.currentPlanData) return;
        
        let textOutput = 'SQL Execution Plan\n';
        textOutput += '==================\n\n';
        
        this.currentPlanData.root_indices.forEach(rootIdx => {
            textOutput += this.nodeToText(this.currentPlanData.nodes, rootIdx, 0);
        });
        
        this.downloadFile(textOutput, 'execution-plan.txt', 'text/plain');
    }

    nodeToText(nodes, nodeIdx, level) {
        const node = nodes[nodeIdx];
        if (!node) return '';
        
        const indent = '  '.repeat(level);
        let output = `${indent}${node.node_type}`;
        
        if (node.relation_name) {
            output += ` on ${node.relation_name}`;
        }
        
        output += `\n${indent}  Cost: ${node.startup_cost?.toFixed(2) || 0}..${node.total_cost?.toFixed(2) || 0}`;
        
        if (node.actual_total_time !== undefined) {
            output += ` | Time: ${node.actual_startup_time?.toFixed(3) || 0}..${node.actual_total_time?.toFixed(3) || 0}ms`;
        }
        
        if (node.actual_rows !== undefined) {
            output += ` | Rows: ${node.actual_rows}`;
        }
        
        output += '\n';
        
        if (node.children && node.children.length > 0) {
            node.children.forEach(childIdx => {
                output += this.nodeToText(nodes, childIdx, level + 1);
            });
        }
        
        return output;
    }

    async copyToClipboard() {
        if (!this.currentPlanData) return;
        
        try {
            const textOutput = this.nodeToText(this.currentPlanData.nodes, this.currentPlanData.root_indices[0], 0);
            await navigator.clipboard.writeText(textOutput);
            
            const btn = document.getElementById('copyPlan');
            const originalText = btn.textContent;
            btn.textContent = 'Copied!';
            btn.style.background = '#28a745';
            
            setTimeout(() => {
                btn.textContent = originalText;
                btn.style.background = '#6c757d';
            }, 2000);
        } catch (err) {
            console.error('Failed to copy to clipboard:', err);
            alert('Failed to copy to clipboard');
        }
    }

    exportAsHtml() {
        if (!this.currentPlanData) return;
        
        const htmlContent = this.generateHtmlReport();
        this.downloadFile(htmlContent, 'execution-plan.html', 'text/html');
    }

    generateHtmlReport() {
        const plan = this.currentPlanData;
        const advisor = this.currentAdvisorAnalysis;
        
        let html = `<!DOCTYPE html>
<html>
<head>
    <title>SQL Execution Plan Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-bottom: 20px; }
        .metric { background: #e9ecef; padding: 15px; border-radius: 6px; text-align: center; }
        .plan-tree { font-family: monospace; background: #f8f9fa; padding: 15px; border-radius: 6px; margin-bottom: 20px; }
        .suggestions { margin-top: 20px; }
        .suggestion { background: #fff3cd; border: 1px solid #ffeaa7; padding: 15px; margin-bottom: 10px; border-radius: 6px; }
        .suggestion.high { background: #f8d7da; border-color: #f5c6cb; }
        .suggestion.medium { background: #fff3cd; border-color: #ffeaa7; }
        .suggestion.low { background: #d1ecf1; border-color: #bee5eb; }
    </style>
</head>
<body>
    <div class="header">
        <h1>SQL Execution Plan Report</h1>
        <p>Generated on ${new Date().toLocaleString()}</p>
    </div>`;

        if (advisor) {
            html += `
    <div class="metrics">
        <div class="metric">
            <h3>${advisor.performance_score}</h3>
            <p>Performance Score</p>
        </div>
        <div class="metric">
            <h3>${advisor.summary.total_suggestions}</h3>
            <p>Total Suggestions</p>
        </div>
        <div class="metric">
            <h3>${advisor.summary.total_cost.toFixed(2)}</h3>
            <p>Total Cost</p>
        </div>
        <div class="metric">
            <h3>${advisor.summary.most_expensive_operation}</h3>
            <p>Most Expensive Operation</p>
        </div>
    </div>`;
        }

        html += `
    <h2>Execution Plan</h2>
    <div class="plan-tree">`;
        
        plan.root_indices.forEach(rootIdx => {
            html += this.nodeToHtml(plan.nodes, rootIdx, 0);
        });
        
        html += `</div>`;

        if (advisor && advisor.suggestions.length > 0) {
            html += `
    <div class="suggestions">
        <h2>Optimization Suggestions</h2>`;
        
            advisor.suggestions.forEach(suggestion => {
                const severityClass = suggestion.severity.toLowerCase();
                html += `
        <div class="suggestion ${severityClass}">
            <h3>${suggestion.title}</h3>
            <p><strong>Type:</strong> ${suggestion.suggestion_type} | <strong>Severity:</strong> ${suggestion.severity}</p>
            <p>${suggestion.description}</p>
            <p><strong>Recommendation:</strong> ${suggestion.recommendation}</p>
            <p><strong>Impact:</strong> ${suggestion.impact}</p>
        </div>`;
            });
            
            html += `</div>`;
        }

        html += `
</body>
</html>`;
        
        return html;
    }

    nodeToHtml(nodes, nodeIdx, level) {
        const node = nodes[nodeIdx];
        if (!node) return '';
        
        const indent = '  '.repeat(level);
        let output = `${indent}${node.node_type}`;
        
        if (node.relation_name) {
            output += ` on ${node.relation_name}`;
        }
        
        output += ` (Cost: ${node.startup_cost?.toFixed(2) || 0}..${node.total_cost?.toFixed(2) || 0})`;
        
        if (node.actual_total_time !== undefined) {
            output += ` (Time: ${node.actual_startup_time?.toFixed(3) || 0}..${node.actual_total_time?.toFixed(3) || 0}ms)`;
        }
        
        if (node.actual_rows !== undefined) {
            output += ` (Rows: ${node.actual_rows})`;
        }
        
        output += '\n';
        
        if (node.children && node.children.length > 0) {
            node.children.forEach(childIdx => {
                output += this.nodeToHtml(nodes, childIdx, level + 1);
            });
        }
        
        return output;
    }

    renderAdvisorSuggestions(analysis) {
        this.currentAdvisorAnalysis = analysis;
        
        if (!analysis || analysis.suggestions.length === 0) {
            this.advisorSection.style.display = 'none';
            return;
        }

        let content = `
            <div class="advisor-summary">
                <div class="score-badge score-${this.getScoreClass(analysis.performance_score)}">
                    Score: ${analysis.performance_score}/100
                </div>
                <p>${analysis.summary.potential_improvement}</p>
            </div>
            <div class="suggestions-list">
        `;

        analysis.suggestions.forEach((suggestion, index) => {
            const severityClass = suggestion.severity.toLowerCase();
            content += `
                <div class="suggestion-item ${severityClass}">
                    <div class="suggestion-header">
                        <span class="suggestion-type">${suggestion.suggestion_type}</span>
                        <span class="suggestion-severity severity-${severityClass}">${suggestion.severity}</span>
                    </div>
                    <h4>${suggestion.title}</h4>
                    <p class="suggestion-description">${suggestion.description}</p>
                    <p class="suggestion-recommendation"><strong>Recommendation:</strong> ${suggestion.recommendation}</p>
                    <p class="suggestion-impact"><strong>Impact:</strong> ${suggestion.impact}</p>
                </div>
            `;
        });

        content += '</div>';
        
        this.advisorContent.innerHTML = content;
        this.advisorSection.style.display = 'block';
    }

    getScoreClass(score) {
        if (score >= 80) return 'good';
        if (score >= 60) return 'medium';
        return 'poor';
    }

    downloadFile(content, filename, contentType) {
        const blob = new Blob([content], { type: contentType });
        const url = URL.createObjectURL(blob);
        
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        
        URL.revokeObjectURL(url);
    }

    toggleTheme() {
        const body = document.body;
        const themeToggle = document.getElementById('themeToggle');
        
        body.classList.toggle('dark-mode');
        
        if (body.classList.contains('dark-mode')) {
            themeToggle.textContent = '☀️';
            localStorage.setItem('theme', 'dark');
        } else {
            themeToggle.textContent = '🌙';
            localStorage.setItem('theme', 'light');
        }
    }

    loadTheme() {
        const savedTheme = localStorage.getItem('theme');
        const body = document.body;
        const themeToggle = document.getElementById('themeToggle');
        
        if (savedTheme === 'dark') {
            body.classList.add('dark-mode');
            themeToggle.textContent = '☀️';
        } else {
            themeToggle.textContent = '🌙';
        }
    }

    calculatePerformanceMetrics(planData) {
        if (!planData || !planData.nodes) return null;
        
        let totalCost = 0;
        let totalTime = 0;
        let totalRows = 0;
        let nodeCount = 0;
        
        planData.nodes.forEach(node => {
            if (node.total_cost) totalCost = Math.max(totalCost, node.total_cost);
            if (node.actual_total_time) totalTime += node.actual_total_time;
            if (node.actual_rows) totalRows += node.actual_rows;
            nodeCount++;
        });
        
        return {
            totalCost: totalCost.toFixed(2),
            totalTime: totalTime.toFixed(3),
            totalRows: totalRows,
            nodeCount: nodeCount
        };
    }

    renderPerformanceMetrics(planData) {
        const metrics = this.calculatePerformanceMetrics(planData);
        if (!metrics) return '';
        
        const costClass = metrics.totalCost > 1000 ? 'cost-high' : 
                         metrics.totalCost > 100 ? 'cost-medium' : 'cost-low';
        
        return `
            <div class="performance-summary">
                <div class="metric-item">
                    <div class="metric-value ${costClass}">${metrics.totalCost}</div>
                    <div class="metric-label">Total Cost</div>
                </div>
                <div class="metric-item">
                    <div class="metric-value">${metrics.totalTime}ms</div>
                    <div class="metric-label">Execution Time</div>
                </div>
                <div class="metric-item">
                    <div class="metric-value">${metrics.totalRows}</div>
                    <div class="metric-label">Rows Processed</div>
                </div>
                <div class="metric-item">
                    <div class="metric-value">${metrics.nodeCount}</div>
                    <div class="metric-label">Plan Nodes</div>
                </div>
            </div>
        `;
    }

    loadHistoryFromStorage() {
        try {
            const history = localStorage.getItem('sqltrace-history');
            return history ? JSON.parse(history) : [];
        } catch (error) {
            console.error('Error loading history from storage:', error);
            return [];
        }
    }

    saveHistoryToStorage() {
        try {
            localStorage.setItem('sqltrace-history', JSON.stringify(this.queryHistory));
        } catch (error) {
            console.error('Error saving history to storage:', error);
        }
    }

    addToHistory(query, planData, advisorAnalysis) {
        const historyItem = {
            id: Date.now().toString(),
            timestamp: new Date().toISOString(),
            query: query,
            planData: planData,
            advisorAnalysis: advisorAnalysis,
            metrics: this.calculatePerformanceMetrics(planData)
        };

        this.queryHistory.unshift(historyItem);
        this.queryHistory = this.queryHistory.slice(0, 50);

        this.saveHistoryToStorage();
        this.renderHistory();
    }

    clearHistory() {
        if (confirm('Are you sure you want to clear all query history?')) {
            this.queryHistory = [];
            this.saveHistoryToStorage();
            this.renderHistory();
            this.exitComparison();
        }
    }

    renderHistory() {
        if (this.queryHistory.length === 0) {
            this.historyList.innerHTML = `
                <div class="empty-history">
                    <p>No queries executed yet. Run a query to see it appear here.</p>
                </div>
            `;
            return;
        }

        const historyHTML = this.queryHistory.map(item => {
            const date = new Date(item.timestamp);
            const timeString = date.toLocaleString();
            const shortQuery = item.query.length > 100 ? 
                item.query.substring(0, 100) + '...' : item.query;

            const isSelected = this.selectedQueries.includes(item.id);
            const comparisonClass = this.comparisonMode ? 
                (isSelected ? 'comparison-selected' : 'comparison-mode') : '';

            return `
                <div class="history-item ${comparisonClass}" data-id="${item.id}">
                    <div class="history-item-header">
                        <div class="history-timestamp">${timeString}</div>
                        ${this.comparisonMode && isSelected ? '<span style="color: #28a745;">✓ Selected</span>' : ''}
                    </div>
                    <div class="history-query">${shortQuery}</div>
                    <div class="history-metrics">
                        ${item.metrics ? `
                            <div class="history-metric">
                                <div class="history-metric-value">${item.metrics.totalCost}</div>
                                <div class="history-metric-label">Cost</div>
                            </div>
                            <div class="history-metric">
                                <div class="history-metric-value">${item.metrics.totalTime}ms</div>
                                <div class="history-metric-label">Time</div>
                            </div>
                            <div class="history-metric">
                                <div class="history-metric-value">${item.metrics.totalRows}</div>
                                <div class="history-metric-label">Rows</div>
                            </div>
                            <div class="history-metric">
                                <div class="history-metric-value">${item.metrics.nodeCount}</div>
                                <div class="history-metric-label">Nodes</div>
                            </div>
                        ` : ''}
                    </div>
                </div>
            `;
        }).join('');

        this.historyList.innerHTML = historyHTML;

        this.historyList.querySelectorAll('.history-item').forEach(item => {
            item.addEventListener('click', (e) => {
                const id = e.currentTarget.getAttribute('data-id');
                if (this.comparisonMode) {
                    this.toggleQuerySelection(id);
                } else {
                    this.loadHistoryItem(id);
                }
            });
        });
    }

    loadHistoryItem(id) {
        const item = this.queryHistory.find(h => h.id === id);
        if (!item) return;

        this.queryInput.value = item.query;

        this.currentPlanData = item.planData;
        this.currentAdvisorAnalysis = item.advisorAnalysis;
        this.renderPlan(item.planData);
        this.renderPerformanceMetrics(item.planData);
        this.renderAdvisorSuggestions(item.advisorAnalysis);
        this.exportSection.style.display = 'block';
    }

    toggleComparison() {
    this.comparisonMode = !this.comparisonMode;
        
    if (this.comparisonMode) {
        this.toggleComparisonBtn.textContent = 'Exit Comparison';
        this.toggleComparisonBtn.classList.add('active');
        this.comparisonSection.style.display = 'block';
            this.selectedQueries = [];
        } else {
            this.exitComparison();
        }
        
        this.renderHistory();
    }

    exitComparison() {
        this.comparisonMode = false;
        this.selectedQueries = [];
        this.toggleComparisonBtn.textContent = 'Compare Mode';
        this.toggleComparisonBtn.classList.remove('active');
        this.comparisonSection.style.display = 'none';
        this.comparisonResults.style.display = 'none';
        this.renderHistory();
    }

    toggleQuerySelection(id) {
        const index = this.selectedQueries.indexOf(id);
        
        if (index === -1) {
            if (this.selectedQueries.length < 2) {
                this.selectedQueries.push(id);
            } else {
                this.selectedQueries[0] = this.selectedQueries[1];
                this.selectedQueries[1] = id;
            }
        } else {
            this.selectedQueries.splice(index, 1);
        }

        this.renderHistory();

        if (this.selectedQueries.length === 2) {
            this.renderComparison();
        } else {
            this.comparisonResults.style.display = 'none';
        }
    }

    renderComparison() {
        if (this.selectedQueries.length !== 2) return;

        const queryA = this.queryHistory.find(h => h.id === this.selectedQueries[0]);
        const queryB = this.queryHistory.find(h => h.id === this.selectedQueries[1]);

        if (!queryA || !queryB) return;

        const comparisonA = this.generateComparisonDetails(queryA, 'A');
        const comparisonB = this.generateComparisonDetails(queryB, 'B');
        const performanceDiff = this.generatePerformanceDiff(queryA.metrics, queryB.metrics);

        document.getElementById('comparisonA').innerHTML = comparisonA;
        document.getElementById('comparisonB').innerHTML = comparisonB;
        
        const comparisonGrid = document.querySelector('.comparison-grid');
        let existingDiff = comparisonGrid.querySelector('.performance-diff');
        if (existingDiff) existingDiff.remove();
        
        comparisonGrid.insertAdjacentHTML('afterend', performanceDiff);
        
        this.comparisonResults.style.display = 'block';
    }

    generateComparisonDetails(item, label) {
        const date = new Date(item.timestamp);
        const shortQuery = item.query.length > 200 ? 
            item.query.substring(0, 200) + '...' : item.query;

        return `
            <div class="comparison-timestamp">${date.toLocaleString()}</div>
            <div class="comparison-query">${shortQuery}</div>
            <div class="comparison-metrics">
                ${item.metrics ? `
                    <div class="comparison-metric">
                        <div class="comparison-metric-value">${item.metrics.totalCost}</div>
                        <div class="comparison-metric-label">Total Cost</div>
                    </div>
                    <div class="comparison-metric">
                        <div class="comparison-metric-value">${item.metrics.totalTime}ms</div>
                        <div class="comparison-metric-label">Execution Time</div>
                    </div>
                    <div class="comparison-metric">
                        <div class="comparison-metric-value">${item.metrics.totalRows}</div>
                        <div class="comparison-metric-label">Rows Processed</div>
                    </div>
                    <div class="comparison-metric">
                        <div class="comparison-metric-value">${item.metrics.nodeCount}</div>
                        <div class="comparison-metric-label">Plan Nodes</div>
                    </div>
                ` : 'No metrics available'}
            </div>
            ${item.advisorAnalysis ? `
                <div class="comparison-advisor">
                    <strong>Performance Score:</strong> ${item.advisorAnalysis.performance_score}/100<br>
                    <strong>Suggestions:</strong> ${item.advisorAnalysis.suggestions.length}
                </div>
            ` : ''}
        `;
    }

    generatePerformanceDiff(metricsA, metricsB) {
        if (!metricsA || !metricsB) {
            return '<div class="performance-diff similar">Unable to compare: Missing metrics data</div>';
        }

        const costA = parseFloat(metricsA.totalCost);
        const costB = parseFloat(metricsB.totalCost);
        const timeA = parseFloat(metricsA.totalTime);
        const timeB = parseFloat(metricsB.totalTime);

        const costDiff = ((costB - costA) / costA * 100).toFixed(1);
        const timeDiff = ((timeB - timeA) / timeA * 100).toFixed(1);

        let diffClass = 'similar';
        let message = '';

        if (Math.abs(costDiff) > 10 || Math.abs(timeDiff) > 10) {
            if (costDiff < 0 && timeDiff < 0) {
                diffClass = 'better';
                message = `Query B is faster: ${Math.abs(costDiff)}% lower cost, ${Math.abs(timeDiff)}% faster execution`;
            } else if (costDiff > 0 && timeDiff > 0) {
                diffClass = 'worse';
                message = `Query B is slower: ${costDiff}% higher cost, ${timeDiff}% slower execution`;
            } else {
                diffClass = 'similar';
                message = `Mixed results: Cost difference ${costDiff}%, Time difference ${timeDiff}%`;
            }
        } else {
            message = 'Queries have similar performance characteristics';
        }

        return `<div class="performance-diff ${diffClass}">${message}</div>`;
    }
}

async function runBenchmark() {
    const query = document.getElementById('sql-query').value.trim();
    if (!query) {
        showError('Please enter a SQL query to benchmark.');
        return;
    }

    const config = {
        warmup_runs: parseInt(document.getElementById('warmup-runs').value) || 2,
        benchmark_runs: parseInt(document.getElementById('benchmark-runs').value) || 5,
        timeout_seconds: parseInt(document.getElementById('timeout-seconds').value) || 30,
        include_execution_plans: true,
        include_advisor_analysis: true
    };

    const resultsDiv = document.getElementById('benchmark-results');
    const button = event.target;
    
    button.disabled = true;
    button.innerHTML = '<span class="loading-spinner"></span> Running Benchmark...';
    resultsDiv.innerHTML = '';
    resultsDiv.classList.remove('show');

    try {
        const response = await fetch('/api/benchmark', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ query, config }),
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        
        if (data.error) {
            showError(`Benchmark failed: ${data.error}`);
            return;
        }

        displayBenchmarkResults(data.result);
        
    } catch (error) {
        console.error('Benchmark error:', error);
        showError('Failed to run benchmark. Please check your connection and try again.');
    } finally {
        button.disabled = false;
        button.innerHTML = '<i class="fas fa-stopwatch"></i> Run Benchmark';
    }
}

async function runComparison() {
    const queryA = document.getElementById('query-a').value.trim();
    const queryB = document.getElementById('query-b').value.trim();
    const labelA = document.getElementById('label-a').value.trim() || 'Query A';
    const labelB = document.getElementById('label-b').value.trim() || 'Query B';

    if (!queryA || !queryB) {
        showError('Please enter both queries to compare.');
        return;
    }

    const config = {
        warmup_runs: parseInt(document.getElementById('warmup-runs').value) || 2,
        benchmark_runs: parseInt(document.getElementById('benchmark-runs').value) || 5,
        timeout_seconds: parseInt(document.getElementById('timeout-seconds').value) || 30,
        include_execution_plans: true,
        include_advisor_analysis: true
    };

    const resultsDiv = document.getElementById('comparison-results');
    const button = event.target;
    
    button.disabled = true;
    button.innerHTML = '<span class="loading-spinner"></span> Comparing Queries...';
    resultsDiv.innerHTML = '';
    resultsDiv.classList.remove('show');

    try {
        const response = await fetch('/api/benchmark/compare', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ 
                query_a: queryA, 
                query_b: queryB, 
                label_a: labelA, 
                label_b: labelB, 
                config 
            }),
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        
        if (data.error) {
            showError(`Comparison failed: ${data.error}`);
            return;
        }

        displayComparisonResults(data.comparison);
        
    } catch (error) {
        console.error('Comparison error:', error);
        showError('Failed to run comparison. Please check your connection and try again.');
    } finally {
        button.disabled = false;
        button.innerHTML = '<i class="fas fa-balance-scale"></i> Compare Queries';
    }
}

function displayBenchmarkResults(result) {
    const resultsDiv = document.getElementById('benchmark-results');
    
    const html = `
        <h4>Benchmark Results</h4>
        <div class="benchmark-stats">
            <div class="stat-card">
                <div class="stat-value">${formatDuration(result.statistics.avg_execution_time)}</div>
                <div class="stat-label">Average Time</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${formatDuration(result.statistics.min_execution_time)}</div>
                <div class="stat-label">Minimum Time</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${formatDuration(result.statistics.max_execution_time)}</div>
                <div class="stat-label">Maximum Time</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${formatDuration(result.statistics.p95_execution_time)}</div>
                <div class="stat-label">95th Percentile</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${result.statistics.successful_runs}</div>
                <div class="stat-label">Successful Runs</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${formatDuration(result.statistics.std_deviation)}</div>
                <div class="stat-label">Std Deviation</div>
            </div>
            ${result.statistics.avg_cost ? `
            <div class="stat-card">
                <div class="stat-value">${result.statistics.avg_cost.toFixed(2)}</div>
                <div class="stat-label">Average Cost</div>
            </div>
            ` : ''}
            ${result.statistics.avg_advisor_score ? `
            <div class="stat-card">
                <div class="stat-value">${result.statistics.avg_advisor_score.toFixed(1)}</div>
                <div class="stat-label">Advisor Score</div>
            </div>
            ` : ''}
        </div>
        <div class="benchmark-query">
            <h5>Query:</h5>
            <pre><code>${escapeHtml(result.query)}</code></pre>
        </div>
    `;
    
    resultsDiv.innerHTML = html;
    resultsDiv.classList.add('show');
}

function displayComparisonResults(comparison) {
    const resultsDiv = document.getElementById('comparison-results');
    
    const improvementClass = comparison.performance_improvement > 5 ? 'positive' : 
                           comparison.performance_improvement < -5 ? 'negative' : 'neutral';
    
    const improvementText = comparison.performance_improvement > 0 ? 
        `${comparison.label_b} is ${comparison.performance_improvement.toFixed(1)}% faster` :
        `${comparison.label_a} is ${Math.abs(comparison.performance_improvement).toFixed(1)}% faster`;
    
    const significanceClass = getSignificanceClass(comparison.statistical_significance);
    const significanceText = getSignificanceText(comparison.statistical_significance);
    
    const html = `
        <h4>Query Comparison Results</h4>
        <div class="comparison-summary">
            <div class="performance-improvement ${improvementClass}">
                ${improvementText}
            </div>
            <div class="significance-badge ${significanceClass}">
                ${significanceText}
            </div>
        </div>
        <div class="benchmark-stats">
            <div class="stat-card">
                <div class="stat-value">${formatDuration(comparison.metrics.avg_time_diff)}</div>
                <div class="stat-label">Time Difference</div>
            </div>
            ${comparison.metrics.cost_diff !== null ? `
            <div class="stat-card">
                <div class="stat-value">${comparison.metrics.cost_diff > 0 ? '+' : ''}${comparison.metrics.cost_diff.toFixed(2)}</div>
                <div class="stat-label">Cost Difference</div>
            </div>
            ` : ''}
            ${comparison.metrics.advisor_score_diff !== null ? `
            <div class="stat-card">
                <div class="stat-value">${comparison.metrics.advisor_score_diff > 0 ? '+' : ''}${comparison.metrics.advisor_score_diff.toFixed(1)}</div>
                <div class="stat-label">Advisor Score Diff</div>
            </div>
            ` : ''}
        </div>
        <div class="comparison-labels">
            <h5>Compared Queries:</h5>
            <p><strong>${comparison.label_a}</strong> vs <strong>${comparison.label_b}</strong></p>
        </div>
    `;
    
    resultsDiv.innerHTML = html;
    resultsDiv.classList.add('show');
}

function formatDuration(duration) {
    const nanos = duration.nanos || 0;
    const secs = duration.secs || 0;
    
    const totalMs = (secs * 1000) + (nanos / 1000000);
    
    if (totalMs < 1) {
        return `${(nanos / 1000).toFixed(0)}μs`;
    } else if (totalMs < 1000) {
        return `${totalMs.toFixed(1)}ms`;
    } else {
        return `${(totalMs / 1000).toFixed(2)}s`;
    }
}

function getSignificanceClass(significance) {
    switch (significance) {
        case 'HighlySignificant': return 'significance-highly-significant';
        case 'Significant': return 'significance-significant';
        case 'MarginallySignificant': return 'significance-marginally-significant';
        default: return 'significance-not-significant';
    }
}

function getSignificanceText(significance) {
    switch (significance) {
        case 'HighlySignificant': return 'Highly Significant';
        case 'Significant': return 'Significant';
        case 'MarginallySignificant': return 'Marginally Significant';
        default: return 'Not Significant';
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

document.addEventListener('DOMContentLoaded', () => {
    new SQLTraceApp();
});
