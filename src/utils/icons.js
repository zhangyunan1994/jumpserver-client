import iconDefault from '../assets/icons/jumpservershellapp.png'
import icon1 from '../assets/icons/jumpservershellapp1.png'
import icon2 from '../assets/icons/jumpservershellapp2.png'
import icon3 from '../assets/icons/jumpservershellapp3.png'
import icon4 from '../assets/icons/jumpservershellapp4.png'

export const iconMap = {
  'jumpservershellapp.png': iconDefault,
  'jumpservershellapp1.png': icon1,
  'jumpservershellapp2.png': icon2,
  'jumpservershellapp3.png': icon3,
  'jumpservershellapp4.png': icon4
}

export function getIconSrc(iconName) {
  return iconMap[iconName] || iconDefault
}

export async function getIconBytes(iconName) {
  const src = iconMap[iconName] || iconDefault
  const resp = await fetch(src)
  const blob = await resp.blob()
  const arrayBuffer = await blob.arrayBuffer()
  return Array.from(new Uint8Array(arrayBuffer))
}
