// Brazilian team crests — stored locally in /public/crests/
// SVG crests from vetores.org + Wikimedia Commons (transparent backgrounds,
// rendered natively in the browser). A few teams keep the old PNGs.

const CREST_MAP: Record<string, string> = {
  // Série A
  'athletico paranaense': 'atlpr.svg',
  'atletico paranaense': 'atlpr.svg',
  'athletico': 'atlpr.svg',
  'atletico mineiro': 'atletico.svg',
  'atletico': 'atletico.svg',
  'bahia': 'bahia.svg',
  'botafogo': 'botafogo.svg',
  'chapecoense': 'chapeco.svg',
  'corinthians': 'corinthians.svg',
  'coritiba': 'coritiba.svg',
  'cruzeiro': 'cruzeiro.svg',
  'flamengo': 'fla.svg',
  'fluminense': 'fluminense.svg',
  'gremio': 'gremio.svg',
  'grêmio': 'gremio.svg',
  'internacional': 'interrs.svg',
  'mirassol': 'mirassol.svg',
  'palmeiras': 'palmeiras.svg',
  'red bull bragantino': 'bragantino.svg',
  'bragantino': 'bragantino.svg',
  'rb bragantino': 'bragantino.svg',
  'remo': 'remo.svg',
  'santos': 'santos.svg',
  'sao paulo': 'saopaulo.svg',
  'são paulo': 'saopaulo.svg',
  'vasco da gama': 'vasco.svg',
  'vasco': 'vasco.svg',
  'vitoria': 'vitoria.svg',
  'vitória': 'vitoria.svg',

  // Série B
  'america mineiro': 'ammg.svg',
  'américa mineiro': 'ammg.svg',
  'avai': 'avai.png',
  'avaí': 'avai.png',
  'ceara': 'ceara.png',
  'ceará': 'ceara.png',
  'crb': 'crb.png',
  'criciuma': 'criciuma.png',
  'criciúma': 'criciuma.png',
  'cuiaba': 'cuiaba_mt.svg',
  'cuiabá': 'cuiaba_mt.svg',
  'goias': 'goias.png',
  'goiás': 'goias.png',
  'fortaleza': 'fortaleza.svg',
  'juventude': 'juventude.png',
  'sport recife': 'sport.png',
  'sport': 'sport.png',
};

function normalize(name: string): string {
  return name
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/\s*-\s*\w{2}$/, '')
    .trim();
}

export function getCrestUrl(name: string): string | null {
  const key = normalize(name);
  const filename = CREST_MAP[key];
  if (filename) {
    return `/crests/${filename}`;
  }
  return null;
}
