import requests
import os

if __name__ == '__main__':
  if not os.path.isdir('data'):
    os.mkdir('data')
  with open('data/gamemaster.json', 'w') as fp:
    r = requests.get('https://raw.githubusercontent.com/ZeChrales/PogoAssets/master/gamemaster/gamemaster.json')
    fp.write(r.text)