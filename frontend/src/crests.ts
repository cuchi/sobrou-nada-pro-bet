// Brazilian team crests — stored locally in /public/crests/
// Downloaded from escudosfc.com.br, normalized to 64x64 PNG with ImageMagick

const CREST_MAP: Record<string, string> = {
  // Série A
  'athletico paranaense': 'atlpr.png',
  'atletico paranaense': 'atlpr.png',
  'athletico': 'atlpr.png',
  'atletico mineiro': 'atletico.png',
  'atletico': 'atletico.png',
  'bahia': 'bahia.png',
  'botafogo': 'botafogo.png',
  'chapecoense': 'chapeco.png',
  'corinthians': 'corinthians.png',
  'coritiba': 'coritiba.png',
  'cruzeiro': 'cruzeiro.png',
  'flamengo': 'fla.png',
  'fluminense': 'fluminense.png',
  'gremio': 'gremio.png',
  'grêmio': 'gremio.png',
  'internacional': 'interrs.png',
  'mirassol': 'mirassol.png',
  'palmeiras': 'palmeiras.png',
  'red bull bragantino': 'bragantino.png',
  'bragantino': 'bragantino.png',
  'rb bragantino': 'bragantino.png',
  'remo': 'remo.png',
  'santos': 'santos.png',
  'sao paulo': 'saopaulo.png',
  'são paulo': 'saopaulo.png',
  'vasco da gama': 'vasco.png',
  'vasco': 'vasco.png',
  'vitoria': 'vitoria.png',
  'vitória': 'vitoria.png',

  // Série B
  'america mineiro': 'ammg.png',
  'américa mineiro': 'ammg.png',
  'avai': 'avai.png',
  'avaí': 'avai.png',
  'ceara': 'ceara.png',
  'ceará': 'ceara.png',
  'crb': 'crb.png',
  'criciuma': 'criciuma.png',
  'criciúma': 'criciuma.png',
  'cuiaba': 'cuiaba_mt.png',
  'cuiabá': 'cuiaba_mt.png',
  'goias': 'goias.png',
  'goiás': 'goias.png',
  'fortaleza': 'fortaleza.png',
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

export function getTeamColor(name: string): string {
  const colors = ['#4a9eff', '#f0c040', '#4ade80', '#f87171', '#a78bfa', '#fb923c', '#2dd4bf', '#f472b6'];
  let hash = 0;
  const n = normalize(name);
  for (let i = 0; i < n.length; i++) hash = n.charCodeAt(i) + ((hash << 5) - hash);
  return colors[Math.abs(hash) % colors.length];
}

export function getInitials(name: string): string {
  return name
    .split(' ')
    .map(w => w[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);
}
