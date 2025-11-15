<template>
  <div class="virtual-table" ref="container">
    <div class="table-wrapper" :style="{ height: containerHeight + 'px' }">
      <div class="spacer-top" :style="{ height: offsetY + 'px' }"></div>
      
      <table class="data-table">
        <thead ref="tableHead" :style="{ transform: `translateY(${scrollTop}px)` }">
          <tr>
            <th 
              v-for="column in columns" 
              :key="column"
              :class="{ 'key-column': column === keyColumn }"
            >
              {{ column }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="(item, index) in visibleItems" 
            :key="getRowKey(item, index)"
            :class="getRowClass(item)"
          >
            <td 
              v-for="column in columns" 
              :key="column"
              :class="getCellClass(item, column)"
            >
              {{ formatCellValue(item, column) }}
            </td>
          </tr>
        </tbody>
      </table>
      
      <div class="spacer-bottom" :style="{ height: bottomSpacerHeight + 'px' }"></div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'VirtualTable',
  props: {
    data: {
      type: Array,
      required: true
    },
    columns: {
      type: Array,
      required: true
    },
    keyColumn: {
      type: String,
      default: 'id'
    },
    viewMode: {
      type: String,
      default: 'single'
    },
    tableName: {
      type: String,
      required: true
    },
    rowHeight: {
      type: Number,
      default: 40 // Average row height in pixels
    },
    overscan: {
      type: Number,
      default: 5 // Number of extra rows to render above/below visible area
    }
  },
  data() {
    return {
      scrollTop: 0,
      containerHeight: 600,
      visibleStart: 0,
      visibleEnd: 20
    };
  },
  computed: {
    totalHeight() {
      return this.data.length * this.rowHeight;
    },
    
    visibleItems() {
      return this.data.slice(this.visibleStart, this.visibleEnd);
    },
    
    offsetY() {
      return this.visibleStart * this.rowHeight;
    },
    
    bottomSpacerHeight() {
      return (this.data.length - this.visibleEnd) * this.rowHeight;
    }
  },
  mounted() {
    this.updateContainerHeight();
    this.$refs.container.addEventListener('scroll', this.onScroll);
    window.addEventListener('resize', this.updateContainerHeight);
  },
  beforeUnmount() {
    this.$refs.container.removeEventListener('scroll', this.onScroll);
    window.removeEventListener('resize', this.updateContainerHeight);
  },
  methods: {
    onScroll() {
      const container = this.$refs.container;
      this.scrollTop = container.scrollTop;
      
      const visibleRowCount = Math.ceil(this.containerHeight / this.rowHeight);
      const scrolledRows = Math.floor(this.scrollTop / this.rowHeight);
      
      this.visibleStart = Math.max(0, scrolledRows - this.overscan);
      this.visibleEnd = Math.min(
        this.data.length,
        scrolledRows + visibleRowCount + this.overscan
      );
      
      // Check if we're near the bottom and need to load more
      if (this.visibleEnd >= this.data.length - 10) {
        this.$emit('load-more');
      }
    },
    
    updateContainerHeight() {
      const container = this.$refs.container;
      if (container) {
        this.containerHeight = container.clientHeight;
        this.onScroll(); // Recalculate visible items
      }
    },
    
    getRowKey(item, index) {
      if (item.type === 'different' && item.data.sourceRow) {
        return `${this.tableName}-${item.type}-${item.data.sourceRow[this.keyColumn]}-${index}`;
      }
      return `${this.tableName}-${item.type}-${item.data[this.keyColumn]}-${index}`;
    },
    
    getRowClass(item) {
      return {
        'row-identical': item.type === 'identical',
        'row-different': item.type === 'different',
        'row-missing': item.type === 'missing',
        'row-extra': item.type === 'extra'
      };
    },
    
    getCellClass(item, column) {
      const classes = {
        'key-cell': column === this.keyColumn
      };
      
      if (item.type === 'different' && item.data.differentColumns) {
        classes['cell-different'] = item.data.differentColumns.includes(column);
      }
      
      return classes;
    },
    
    formatCellValue(item, column) {
      let value;
      
      if (item.type === 'different') {
        // For different rows, show source value in single view, or based on viewMode
        value = this.viewMode === 'sideBySide' 
          ? item.data.sourceRow[column]
          : item.data.sourceRow[column];
      } else {
        value = item.data[column];
      }
      
      if (value === null || value === undefined) {
        return 'NULL';
      }
      if (typeof value === 'object') {
        return JSON.stringify(value);
      }
      return String(value);
    }
  }
};
</script>

<style scoped>
.virtual-table {
  width: 100%;
  height: 600px;
  overflow: auto;
  position: relative;
  background: white;
}

.table-wrapper {
  position: relative;
  width: 100%;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9em;
}

.data-table thead {
  position: sticky;
  top: 0;
  z-index: 20;
  background: white;
}

.data-table th {
  padding: 12px;
  text-align: left;
  background: #f8f9fa;
  border-bottom: 2px solid #dee2e6;
  font-weight: 600;
  color: #495057;
  white-space: nowrap;
  position: sticky;
  top: 0;
  z-index: 10;
}

.data-table th.key-column {
  background: #e3f2fd;
  color: #1976d2;
  border-right: 2px solid #2196f3;
}

.data-table td {
  padding: 10px 12px;
  border-bottom: 1px solid #e9ecef;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}

.data-table td.key-cell {
  background: #f0f8ff;
  font-weight: 600;
  color: #1976d2;
  border-right: 1px solid #2196f3;
}

/* Row Type Styling */
.row-identical {
  background: #f8fff8;
}

.row-identical:hover {
  background: #e8f5e8;
}

.row-different {
  background: #fffef5;
}

.row-different:hover {
  background: #fef3c7;
}

.row-missing {
  background: #fff5f5;
}

.row-missing:hover {
  background: #fee2e2;
}

.row-extra {
  background: #f5f8ff;
}

.row-extra:hover {
  background: #e0e7ff;
}

.cell-different {
  background: #fed7aa !important;
  font-weight: 600 !important;
  color: #d97706 !important;
}

/* Scrollbar Styling */
.virtual-table::-webkit-scrollbar {
  width: 12px;
  height: 12px;
}

.virtual-table::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.virtual-table::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 6px;
}

.virtual-table::-webkit-scrollbar-thumb:hover {
  background: #555;
}

/* Spacers for virtual scrolling */
.spacer-top,
.spacer-bottom {
  width: 100%;
}
</style>