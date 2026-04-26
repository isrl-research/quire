import { ref } from 'vue'

export interface BibEntry {
  key: string
  entryType: string
  title?: string
  authors?: string
  year?: string
  journal?: string
  doi?: string
  abstractText?: string
  url?: string
  volume?: string
  issue?: string
  pages?: string
  publisher?: string
  booktitle?: string
  edition?: string
  month?: string
  keywords?: string
  note?: string
  isbn?: string
  issn?: string
  number?: string
  institution?: string
}

export interface DocAuthor {
  name: string
  orcid: string
  title: string
  affiliation: string
}

// Module-level singletons — shared across all components without Pinia
const filePath = ref<string | null>(null)
const bibPath = ref<string | null>(null)
const docTitle = ref('Allergen Labelling Study')
const docSubtitle = ref('A Cross-Sectional Study of FSSAI Compliance and Consumer Risk Communication')
const docStatus = ref('Working Draft')
const docDate = ref('April 2026')
const docAuthors = ref<DocAuthor[]>([
  { name: 'Sharma, R.', orcid: '', title: 'PhD Candidate', affiliation: 'Food Policy Studies, IIT Madras' },
])
const isDirty = ref(false)
const lastSaved = ref<Date | null>(null)
const citations = ref<BibEntry[]>([
  {
    key: 'popova2022',
    entryType: 'article',
    title: 'Allergen Labelling Compliance in Food Manufacturing: A Systematic Review',
    authors: 'Popova, M., Chen, L., & Okafor, R.',
    year: '2022',
    journal: 'Food Control',
    doi: '10.1016/j.foodcont.2022.108940',
    abstractText:
      'A systematic review examining allergen labelling compliance across 14 countries. Of 847 products sampled, 34% showed discrepancies between declared allergens and actual ingredient lists, with peanut and tree nut labelling showing the highest non-compliance rates. The study identifies inconsistent definition of precautionary labelling as a primary driver of consumer confusion.',
  },
  {
    key: 'fda2021',
    entryType: 'report',
    title: 'Food Allergen Labeling and Consumer Protection Act: Compliance Report 2021',
    authors: 'U.S. Food and Drug Administration',
    year: '2021',
    journal: 'FDA Technical Report',
    doi: 'FDA-TR-2021-0042',
    abstractText:
      'Annual compliance report on FALCPA implementation, documenting a 1-in-8 consumer experience with allergic reactions attributable to labelling failures in packaged foods over a 24-month surveillance period. Survey conducted across n=12,400 households with at least one member reporting a food allergy.',
  },
  {
    key: 'hadley2019',
    entryType: 'article',
    title: 'Hidden Allergens and Precautionary Labelling: Consumer Understanding and Risk',
    authors: 'Hadley, C. & King, J.',
    year: '2019',
    journal: 'J Allergy Clin Immunol',
    doi: '10.1016/j.jaci.2018.11.025',
    abstractText:
      'Cross-sectional study of consumer comprehension of precautionary allergen labels (PAL) across a 2019 baseline cohort. Finds significant variation in PAL interpretation by education level and prior allergy diagnosis. 67% of respondents misinterpreted "may contain" as indicating lower risk than a direct ingredient declaration.',
  },
])
const citationOrder = ref<string[]>(['popova2022', 'fda2021', 'hadley2019'])

export function useDocument() {
  return {
    filePath,
    bibPath,
    docTitle,
    docSubtitle,
    docStatus,
    docDate,
    docAuthors,
    isDirty,
    lastSaved,
    citations,
    citationOrder,
  }
}
