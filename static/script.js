// Main application JavaScript for SQLTrace

class SQLTraceApp {
    constructor() {
        this.queryInput = document.getElementById('queryInput');
        this.executeBtn = document.getElementById('executeBtn');
        this.errorContainer = document.getElementById('errorContainer');
        this.errorText = document.getElementById('errorText');
        this.planContainer = document.getElementById('planContainer');
        this.exportSection = document.getElementById('exportSection');
        this.currentPlanData = null;
        
        this.init();
    }

    init() {
        this.executeBtn.addEventListener('click', () => this.executeQuery());
        this.queryInput.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'Enter') {
                this.executeQuery();
            }
        });

        // Add event listeners for example queries
        document.querySelectorAll('.example-query').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const query = e.target.getAttribute('data-query');
                this.queryInput.value = query;
                this.executeQuery();
            });
        });

        // Add event listeners for export buttons
        document.getElementById('exportJson')?.addEventListener('click', () => this.exportAsJson());
        document.getElementById('exportText')?.addEventListener('click', () => this.exportAsText());
        document.getElementById('copyPlan')?.addEventListener('click', () => this.copyToClipboard());

        // Add theme toggle functionality
        document.getElementById('themeToggle')?.addEventListener('click', () => this.toggleTheme());
        this.loadTheme();
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
            }
        } catch (error) {
            this.showError(`Network error: ${error.message}`);
            this.showEmptyState();
        } finally {
            this.setLoading(false);
        }
    }

    setLoading(loading) {
        this.executeBtn.disabled = loading;
        this.executeBtn.querySelector('.btn-text').textContent = loading ? 'Analyzing...' : 'Analyze Query';
        this.executeBtn.querySelector('.btn-spinner').style.display = loading ? 'inline-block' : 'none';
        
        // Add loading class to main container
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

        // Store current plan data for export functionality
        this.currentPlanData = planData;

        // Create performance metrics summary
        const performanceMetrics = this.renderPerformanceMetrics(planData);
        
        const planTree = document.createElement('div');
        planTree.className = 'plan-tree';
        
        // Render root nodes
        planData.root_indices.forEach(rootIdx => {
            this.renderNode(planTree, planData.nodes, rootIdx, 0);
        });

        this.planContainer.innerHTML = performanceMetrics;
        this.planContainer.appendChild(planTree);
        
        // Show export section
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

        // Create node content
        const nodeContent = this.createNodeContent(node, level, hasChildren, node.expanded);
        nodeElement.innerHTML = nodeContent;

        // Add click handler for expandable nodes
        if (hasChildren) {
            nodeElement.addEventListener('click', (e) => {
                e.stopPropagation();
                this.toggleNode(nodeElement, nodes, nodeIdx);
            });
        }

        container.appendChild(nodeElement);

        // Render children if expanded
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

        // Build details
        const details = [];
        
        if (planNode.total_cost !== undefined) {
            details.push(`<span class="plan-node-cost">Cost: ${planNode.startup_cost?.toFixed(2) || 0}..${planNode.total_cost.toFixed(2)}</span>`);
        }
        
        if (planNode.actual_total_time !== undefined) {
            details.push(`<span class="plan-node-time">Time: ${planNode.actual_startup_time?.toFixed(3) || 0}..${planNode.actual_total_time.toFixed(3)}ms</span>`);
        }
        
        // Note: plan_rows field removed from structure
        
        if (planNode.actual_rows !== undefined) {
            details.push(`<span class="plan-node-rows">Actual Rows: ${planNode.actual_rows}</span>`);
        }

        // Add condition information from extra field
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
        
        // Re-render the entire plan to reflect the change
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

    // Export functionality
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
        
        // Add children
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
            
            // Show feedback
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

    // Theme functionality
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

    // Performance metrics calculation
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
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new SQLTraceApp();
});
