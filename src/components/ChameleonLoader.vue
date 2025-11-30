<!-- Schema Color Loader -->
<template>
  <div class="rainbow-loader">
    <div class="spinner-container">
      <svg class="circle-svg" viewBox="0 0 100 100">
        <!-- 
          Donut Chart Loader
          Radius: 40
          Circumference: ~251.3
          5 Segments
          Gap: ~10 units
          Segment Length: ~40 units
        -->
        
        <!-- Red segment (Deleted) -->
        <circle 
          class="segment segment-red" 
          cx="50" cy="50" r="40" 
        />
        
        <!-- Orange segment (Modified) -->
        <circle 
          class="segment segment-orange" 
          cx="50" cy="50" r="40" 
        />
        
        <!-- Green segment (Added) -->
        <circle 
          class="segment segment-green" 
          cx="50" cy="50" r="40" 
        />
        
        <!-- Gray segment (Unchanged) -->
        <circle 
          class="segment segment-gray" 
          cx="50" cy="50" r="40" 
        />
        
        <!-- Primary segment (App Color) -->
        <circle 
          class="segment segment-primary" 
          cx="50" cy="50" r="40" 
        />
      </svg>
    </div>
    <div v-if="showText" class="loading-text">{{ text }}</div>
  </div>
</template>

<script setup lang="ts">
defineProps({
  showText: {
    type: Boolean,
    default: true
  },
  text: {
    type: String,
    default: 'Loading...'
  }
});
</script>

<style scoped>
.rainbow-loader {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.spinner-container {
  position: relative;
  width: 80px;
  height: 80px;
}

.circle-svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg); /* Start from top */
}

.segment {
  fill: none;
  stroke-width: 12; /* Thickness of the donut */
  stroke-linecap: butt; /* Sharp ends for segments */
  stroke-dasharray: 42 209.3; /* Segment length 42, Gap/Remaining 209.3 */
  opacity: 0;
  transform-origin: center;
  animation: loadSegment 1.5s ease-in-out infinite;
}

/* 
  Rotations for 5 segments (72 degrees each)
  0, 72, 144, 216, 288
*/

.segment-red {
  stroke: #ef4444;
  transform: rotate(0deg);
  animation-delay: 0s;
}

.segment-orange {
  stroke: #f59e0b;
  transform: rotate(72deg);
  animation-delay: 0.3s;
}

.segment-green {
  stroke: #10b981;
  transform: rotate(144deg);
  animation-delay: 0.6s;
}

.segment-gray {
  stroke: #9ca3af;
  transform: rotate(216deg);
  animation-delay: 0.9s;
}

.segment-primary {
  stroke: #6366f1;
  transform: rotate(288deg);
  animation-delay: 1.2s;
}

@keyframes loadSegment {
  0% {
    opacity: 0;
    stroke-dasharray: 0 251.3; /* Start empty */
  }
  10% {
    opacity: 1;
  }
  20% {
    stroke-dasharray: 42 209.3; /* Full segment size */
  }
  80% {
    opacity: 1;
    stroke-dasharray: 42 209.3;
  }
  100% {
    opacity: 0;
    stroke-dasharray: 42 209.3;
  }
}

.loading-text {
  font-size: 14px;
  color: var(--text-muted);
  font-weight: 500;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}
</style>
