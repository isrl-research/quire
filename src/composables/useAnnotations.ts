import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Annotation {
  id: number
  itemId: number
  attachmentId: number
  page: number
  annType: string
  color: string
  selectedText?: string
  noteText?: string
  positionJson: string
  createdAt: number
}

export interface AnnotationWithSource {
  id: number
  itemId: number
  attachmentId: number
  page: number
  annType: string
  color: string
  selectedText?: string
  noteText?: string
  createdAt: number
  itemTitle?: string
  itemAuthors?: string
  itemYear?: string
  itemKey: string
  attachmentFileName: string
}

export interface AnnotationInput {
  itemId: number
  attachmentId: number
  page: number
  annType: string
  color: string
  selectedText?: string
  noteText?: string
  positionJson: string
}

export function useAnnotations() {
  const annotations = ref<Annotation[]>([])

  async function loadAnnotations(attachmentId: number): Promise<void> {
    annotations.value = await invoke<Annotation[]>(
      'get_annotations_for_attachment', { attachmentId }
    )
  }

  async function createAnnotation(input: AnnotationInput): Promise<Annotation> {
    const ann = await invoke<Annotation>('create_annotation', { input })
    annotations.value.push(ann)
    return ann
  }

  async function deleteAnnotation(id: number): Promise<void> {
    await invoke('delete_annotation', { id })
    annotations.value = annotations.value.filter(a => a.id !== id)
  }

  async function updateAnnotationNote(id: number, noteText: string): Promise<void> {
    await invoke('update_annotation_note', { id, noteText })
    const idx = annotations.value.findIndex(a => a.id === id)
    if (idx !== -1) annotations.value[idx] = { ...annotations.value[idx], noteText }
  }

  return { annotations, loadAnnotations, createAnnotation, deleteAnnotation, updateAnnotationNote }
}

// Module-level singleton for the cross-source annotations list
const allAnnotations = ref<AnnotationWithSource[]>([])
const allAnnotationsLoading = ref(false)

export function useAllAnnotations() {
  async function loadAll(): Promise<void> {
    allAnnotationsLoading.value = true
    try {
      allAnnotations.value = await invoke<AnnotationWithSource[]>('get_all_annotations')
    } finally {
      allAnnotationsLoading.value = false
    }
  }

  async function deleteAndRefresh(id: number): Promise<void> {
    await invoke('delete_annotation', { id })
    allAnnotations.value = allAnnotations.value.filter(a => a.id !== id)
  }

  async function updateNoteAndRefresh(id: number, noteText: string): Promise<void> {
    await invoke('update_annotation_note', { id, noteText })
    const idx = allAnnotations.value.findIndex(a => a.id === id)
    if (idx !== -1) allAnnotations.value[idx] = { ...allAnnotations.value[idx], noteText }
  }

  return {
    allAnnotations,
    allAnnotationsLoading,
    loadAll,
    deleteAndRefresh,
    updateNoteAndRefresh,
  }
}
