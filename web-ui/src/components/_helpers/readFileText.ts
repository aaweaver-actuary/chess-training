export const readFileText = (file: File): Promise<string> => {
  if (typeof file.text === 'function') {
    return file.text();
  }

  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result;
      resolve(typeof result === 'string' ? result : '');
    };
    reader.onerror = () => {
      /* c8 ignore next */
      reject(reader.error ?? new Error('Unable to read PGN file.'));
    };
    reader.readAsText(file);
  });
};
