import pandas as pd
import os

class ExcelManager:
    def __init__(self):
        self.current_file = None
        self.df = None

    def load_excel(self, file_path):
        """
        Loads the excel file and ensures it has at least 3 columns: ID, Name, Grade.
        Returns a list of dictionaries with the student data.
        """
        try:
            self.current_file = file_path
            # Leemos el excel
            self.df = pd.read_excel(file_path)
            
            # Verificamos que tenga al menos 3 columnas
            if len(self.df.columns) >= 3:
                cols = list(self.df.columns)
                self.id_col = cols[0]
                self.name_col = cols[1]
                self.grade_col = cols[2]
                
                # Renombrar internamente para facilitar el manejo
                self.df.rename(columns={self.id_col: 'ID', self.name_col: 'Nombre', self.grade_col: 'Nota'}, inplace=True)
                
                return self.df.to_dict('records')
            else:
                raise ValueError("El Excel debe tener al menos 3 columnas: ID, Nombre, Nota")
        except Exception as e:
            print(f"Error loading Excel: {e}")
            return None

    def update_grade(self, student_id, new_grade):
        """Updates the grade in the dataframe."""
        if self.df is not None:
            self.df.loc[self.df['ID'] == student_id, 'Nota'] = new_grade

    def export_excel(self, save_path):
        """Exports the current dataframe to a new Excel file."""
        if self.df is not None and save_path:
            try:
                # Restaurar los nombres originales antes de exportar
                export_df = self.df.rename(columns={'ID': self.id_col, 'Nombre': self.name_col, 'Nota': self.grade_col})
                export_df.to_excel(save_path, index=False)
                return True
            except Exception as e:
                print(f"Error exporting Excel: {e}")
                return False
        return False
