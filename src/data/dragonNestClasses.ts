import swordmaster from '../assets/classes/jobs/swordmaster.png'
import moonlord from '../assets/classes/jobs/moonlord.png'
import destroyer from '../assets/classes/jobs/destroyer.png'
import barbarian from '../assets/classes/jobs/barbarian.png'
import sniper from '../assets/classes/jobs/sniper.png'
import artillery from '../assets/classes/jobs/artillery.png'
import tempest from '../assets/classes/jobs/tempest.png'
import windwalker from '../assets/classes/jobs/windwalker.png'
import saleana from '../assets/classes/jobs/saleana.png'
import elestra from '../assets/classes/jobs/elestra.png'
import smasher from '../assets/classes/jobs/smasher.png'
import majesty from '../assets/classes/jobs/majesty.png'
import guardian from '../assets/classes/jobs/guardian.png'
import crusader from '../assets/classes/jobs/crusader.png'
import inquisitor from '../assets/classes/jobs/inquisitor.png'
import saint from '../assets/classes/jobs/saint.png'
import shootingStar from '../assets/classes/jobs/shooting_star.png'
import gearMaster from '../assets/classes/jobs/gear_master.png'
import adept from '../assets/classes/jobs/adept.png'
import physician from '../assets/classes/jobs/physician.png'
import soulEater from '../assets/classes/jobs/soul_eater.png'
import darkSummoner from '../assets/classes/jobs/dark_summoner.png'
import bladeDancer from '../assets/classes/jobs/blade_dancer.png'
import spiritDancer from '../assets/classes/jobs/spirit_dancer.png'
import raven from '../assets/classes/jobs/raven.png'
import ripper from '../assets/classes/jobs/ripper.png'
import abyssWalker from '../assets/classes/jobs/abyss_walker.png'
import lightFury from '../assets/classes/jobs/light_fury.png'
import darkAvenger from '../assets/classes/jobs/dark_avenger.png'

export interface DragonNestClass {
  key: string
  base: string
  name: string
  icon: string
  order: number
}

export const dragonNestClasses: DragonNestClass[] = [
  { key: 'swordmaster', base: '战士', name: '剑皇', icon: swordmaster, order: 10 },
  { key: 'moonlord', base: '战士', name: '月之领主', icon: moonlord, order: 11 },
  { key: 'destroyer', base: '战士', name: '毁灭者', icon: destroyer, order: 12 },
  { key: 'barbarian', base: '战士', name: '狂战士', icon: barbarian, order: 13 },
  { key: 'sniper', base: '弓箭手', name: '狙翎', icon: sniper, order: 20 },
  { key: 'artillery', base: '弓箭手', name: '魔羽', icon: artillery, order: 21 },
  { key: 'tempest', base: '弓箭手', name: '影舞者', icon: tempest, order: 22 },
  { key: 'windwalker', base: '弓箭手', name: '风行者', icon: windwalker, order: 23 },
  { key: 'saleana', base: '魔法师', name: '火舞', icon: saleana, order: 30 },
  { key: 'elestra', base: '魔法师', name: '冰灵', icon: elestra, order: 31 },
  { key: 'smasher', base: '魔法师', name: '时空领主', icon: smasher, order: 32 },
  { key: 'majesty', base: '魔法师', name: '黑暗女王', icon: majesty, order: 33 },
  { key: 'guardian', base: '牧师', name: '圣骑士', icon: guardian, order: 40 },
  { key: 'crusader', base: '牧师', name: '十字军', icon: crusader, order: 41 },
  { key: 'inquisitor', base: '牧师', name: '雷神', icon: inquisitor, order: 42 },
  { key: 'saint', base: '牧师', name: '圣徒', icon: saint, order: 43 },
  { key: 'shooting_star', base: '学者', name: '重炮手', icon: shootingStar, order: 50 },
  { key: 'gear_master', base: '学者', name: '机械大师', icon: gearMaster, order: 51 },
  { key: 'adept', base: '学者', name: '炼金圣士', icon: adept, order: 52 },
  { key: 'physician', base: '学者', name: '药剂师', icon: physician, order: 53 },
  { key: 'soul_eater', base: '舞娘', name: '噬魂者', icon: soulEater, order: 60 },
  { key: 'dark_summoner', base: '舞娘', name: '黑暗萨满', icon: darkSummoner, order: 61 },
  { key: 'blade_dancer', base: '舞娘', name: '刀锋舞者', icon: bladeDancer, order: 62 },
  { key: 'spirit_dancer', base: '舞娘', name: '灵魂舞者', icon: spiritDancer, order: 63 },
  { key: 'raven', base: '刺客', name: '影', icon: raven, order: 70 },
  { key: 'ripper', base: '刺客', name: '烈', icon: ripper, order: 71 },
  { key: 'abyss_walker', base: '刺客', name: '暗', icon: abyssWalker, order: 72 },
  { key: 'light_fury', base: '刺客', name: '曜', icon: lightFury, order: 73 },
  { key: 'dark_avenger', base: '外传', name: '黑暗复仇者', icon: darkAvenger, order: 80 },
]

export function getClassByKey(key?: string | null) {
  return dragonNestClasses.find((item) => item.key === key) ?? null
}

export function getClassOrder(key?: string | null) {
  return getClassByKey(key)?.order ?? 999
}
