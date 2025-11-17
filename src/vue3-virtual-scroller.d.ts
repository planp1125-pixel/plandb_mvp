declare module 'vue3-virtual-scroller' {
  import { DefineComponent } from 'vue'

  export const RecycleScroller: DefineComponent<{
    items: any[]
    itemSize: number
    buffer?: number
    keyField?: string
  }>

  export const DynamicScroller: DefineComponent<any>
  export const DynamicScrollerItem: DefineComponent<any>
}
