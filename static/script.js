// Main application JavaScript for SQLTrace

class SQLTraceApp {
    constructor() {
        this.queryInput = document.getElementById('queryInput');
        this.executeBtn = document.getElementById('executeBtn');
        this.errorContainer = document.getElementById('errorContainer');
        this.errorText = document.getElementById('errorText');
        this.planContainer = document.getElementById('planContainer');
        
        this.init();
    }

    init() {
        this.executeBtn.addEventListener('click', () => this.executeQuery());
        this.queryInput.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'Enter') {
                this.executeQuery();
            }
        });
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
        const btnText = this.executeBtn.querySelector('.btn-text');
        const btnSpinner = this.executeBtn.querySelector('.btn-spinner');
        
        this.executeBtn.disabled = loading;
        
        if (loading) {
            btnText.style.display = 'none';
            btnSpinner.style.display = 'inline';
        } else {
            btnText.style.display = 'inline';
            btnSpinner.style.display = 'none';
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

        const planTree = document.createElement('div');
        planTree.className = 'plan-tree';
        
        // Render root nodes
        planData.root_indices.forEach(rootIdx => {
            this.renderNode(planTree, planData.nodes, rootIdx, 0);
        });

        this.planContainer.innerHTML = '';
        this.planContainer.appendChild(planTree);
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
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new SQLTraceApp();
});
